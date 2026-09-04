#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use clap::{Parser, ValueEnum};
use colored::Colorize;
use config::Config;
use question::{Answer, Question};
use reqwest::blocking::{Client, Response};
use serde_json::{json, Value};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{self, BufRead, BufReader, IsTerminal, Write};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Description of the command to execute
    #[clap(required = true)]
    prompt: String,

    /// Run the generated program without asking for confirmation
    #[clap(short = 'y', long)]
    force: bool,

    /// Override the gateway model for this request
    #[clap(long, value_name = "MODEL")]
    model: Option<String>,

    /// Override reasoning effort for this request
    #[clap(
        long = "reasoning-effort",
        visible_alias = "reasoning",
        value_enum,
        value_name = "EFFORT"
    )]
    reasoning_effort: Option<ReasoningEffort>,
}

#[derive(Clone, Debug, ValueEnum)]
enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl ReasoningEffort {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
        }
    }
}

struct ShellStreamPreview<'a> {
    syntax_set: &'a SyntaxSet,
    syntax: &'a SyntaxReference,
    theme: &'a Theme,
    highlighter: HighlightLines<'a>,
    complete_lines: Vec<String>,
    partial_line: String,
    partial_visible: bool,
    is_terminal: bool,
    terminal_width: usize,
    partial_rows: usize,
}

struct WaitIndicator {
    done: Arc<AtomicBool>,
    first_content: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    enabled: bool,
}

impl WaitIndicator {
    fn start(model: &str, reasoning_effort: &str) -> Self {
        let enabled = io::stderr().is_terminal();
        let done = Arc::new(AtomicBool::new(false));
        let first_content = Arc::new(AtomicBool::new(false));

        if !enabled {
            return Self {
                done,
                first_content,
                handle: None,
                enabled,
            };
        }

        let done_thread = Arc::clone(&done);
        let first_content_thread = Arc::clone(&first_content);
        let model = model.to_string();
        let reasoning_effort = reasoning_effort.to_string();
        let handle = thread::spawn(move || {
            let started_at = Instant::now();
            loop {
                if done_thread.load(Ordering::Relaxed)
                    || first_content_thread.load(Ordering::Relaxed)
                {
                    break;
                }

                let elapsed = started_at.elapsed().as_secs();
                if elapsed > 0 {
                    eprint!(
                        "\r\x1b[2KWaiting for first token from {model} / {reasoning_effort}... {elapsed}s"
                    );
                    let _ = io::stderr().flush();
                }

                thread::sleep(Duration::from_millis(250));
            }
        });

        Self {
            done,
            first_content,
            handle: Some(handle),
            enabled,
        }
    }

    fn mark_first_content(&self) {
        if !self.first_content.swap(true, Ordering::Relaxed) {
            self.clear();
        }
    }

    fn finish(&mut self) {
        self.done.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        self.clear();
    }

    fn clear(&self) {
        if self.enabled {
            eprint!("\r\x1b[2K");
            let _ = io::stderr().flush();
        }
    }
}

impl<'a> ShellStreamPreview<'a> {
    fn new(syntax_set: &'a SyntaxSet, syntax: &'a SyntaxReference, theme: &'a Theme) -> Self {
        Self {
            syntax_set,
            syntax,
            theme,
            highlighter: HighlightLines::new(syntax, theme),
            complete_lines: Vec::new(),
            partial_line: String::new(),
            partial_visible: false,
            is_terminal: io::stdout().is_terminal(),
            terminal_width: terminal_width(),
            partial_rows: 0,
        }
    }

    fn push_str(&mut self, content: &str) -> Result<(), String> {
        for segment in content.split_inclusive('\n') {
            self.partial_line.push_str(segment);
            if segment.ends_with('\n') {
                self.commit_partial_line()?;
            } else {
                self.render_partial_line()?;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), String> {
        if !self.partial_line.is_empty() {
            self.commit_partial_line()?;
        }

        writeln!(io::stdout()).map_err(|e| format!("Failed to finish stream preview: {e}"))
    }

    fn commit_partial_line(&mut self) -> Result<(), String> {
        let line = std::mem::take(&mut self.partial_line);
        self.clear_partial_line()?;
        let highlighted = highlighted_line(&mut self.highlighter, self.syntax_set, &line)?;

        print!("{highlighted}");
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stream preview: {e}"))?;

        self.complete_lines.push(line);
        self.partial_visible = false;
        Ok(())
    }

    fn render_partial_line(&mut self) -> Result<(), String> {
        if self.partial_line.is_empty() || !self.is_terminal {
            return Ok(());
        }

        self.clear_partial_line()?;
        let highlighted = self.highlight_partial_line()?;

        print!("{highlighted}");
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to flush stream preview: {e}"))?;

        self.partial_visible = true;
        self.partial_rows = wrapped_rows(&self.partial_line, self.terminal_width);
        Ok(())
    }

    fn highlight_partial_line(&self) -> Result<String, String> {
        let mut highlighter = HighlightLines::new(self.syntax, self.theme);
        for line in &self.complete_lines {
            highlighter
                .highlight_line(line, self.syntax_set)
                .map_err(|e| format!("Failed to highlight stream preview: {e}"))?;
        }
        highlighted_line(&mut highlighter, self.syntax_set, &self.partial_line)
    }

    fn clear_partial_line(&self) -> Result<(), String> {
        if !self.partial_visible || !self.is_terminal {
            return Ok(());
        }

        print!("\r\x1b[2K");
        for _ in 1..self.partial_rows {
            print!("\x1b[1A\r\x1b[2K");
        }
        io::stdout()
            .flush()
            .map_err(|e| format!("Failed to clear stream preview: {e}"))
    }
}

fn highlighted_line(
    highlighter: &mut HighlightLines<'_>,
    syntax_set: &SyntaxSet,
    line: &str,
) -> Result<String, String> {
    let ranges = highlighter
        .highlight_line(line, syntax_set)
        .map_err(|e| format!("Failed to highlight stream preview: {e}"))?;
    Ok(format!(
        "{}\x1b[0m",
        as_24_bit_terminal_escaped(&ranges, false)
    ))
}

fn terminal_width() -> usize {
    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > 0)
        .unwrap_or(80)
}

fn wrapped_rows(line: &str, terminal_width: usize) -> usize {
    line.chars().count().saturating_sub(1) / terminal_width + 1
}

fn generate_code(
    prompt: &str,
    config: &Config,
    preview: &mut ShellStreamPreview<'_>,
    wait_indicator: &WaitIndicator,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))?;

    let response = client
        .post(&config.chat_completions_url)
        .json(&json!({
            "model": &config.model,
            "reasoning_effort": &config.reasoning_effort,
            "stream": true,
            "prompt_cache_key": &config.prompt_cache_key,
            "messages": [
                {
                    "role": "system",
                    "content": &config.system_prompt
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .header("Authorization", format!("Bearer {}", config.api_token))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .send()
        .map_err(|e| format!("API request failed: {e}"))?;

    let status = response.status();
    if status.is_client_error() {
        let error_message = parse_error_response(response);
        return Err(format!("API error: \"{error_message}\""));
    } else if status.is_server_error() {
        return Err(format!(
            "UOS AI Gateway is currently experiencing problems. Status code: {status}"
        ));
    }

    let mut reader = BufReader::new(response);
    let mut event_data = String::new();
    let mut code = String::new();
    let mut stream_event_count = 0_u32;

    loop {
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read streaming API response: {e}"))?;

        if bytes_read == 0 {
            if !event_data.trim().is_empty() {
                stream_event_count += 1;
                handle_stream_event(&event_data, &mut code, preview, wait_indicator)?;
            }
            break;
        }

        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            if !event_data.trim().is_empty() {
                stream_event_count += 1;
            }
            if handle_stream_event(&event_data, &mut code, preview, wait_indicator)? {
                break;
            }
            event_data.clear();
        } else if let Some(data) = line.strip_prefix("data:") {
            if !event_data.is_empty() {
                event_data.push('\n');
            }
            event_data.push_str(data.trim_start());
        }
    }

    let code = code.trim().to_string();
    if code.is_empty() {
        if stream_event_count == 0 {
            return Err(format!(
                "Gateway returned an empty stream for {} / {}. Retry the request or use a lower reasoning effort.",
                config.model, config.reasoning_effort
            ));
        }
        return Err(format!(
            "Gateway stream ended without script content for {} / {}. Retry the request or use a lower reasoning effort.",
            config.model, config.reasoning_effort
        ));
    }

    Ok(code)
}

fn parse_error_response(response: Response) -> String {
    let body = response.text().unwrap_or_default();
    let parsed = serde_json::from_str::<Value>(&body).ok();

    parsed
        .as_ref()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.get("detail"))
                .and_then(Value::as_str)
        })
        .unwrap_or("Unknown error")
        .to_string()
}

fn handle_stream_event(
    event_data: &str,
    code: &mut String,
    preview: &mut ShellStreamPreview<'_>,
    wait_indicator: &WaitIndicator,
) -> Result<bool, String> {
    let payload = event_data.trim();
    if payload.is_empty() {
        return Ok(false);
    }
    if payload == "[DONE]" {
        return Ok(true);
    }

    let event = serde_json::from_str::<Value>(payload)
        .map_err(|e| format!("Failed to parse streaming API response: {e}"))?;

    if let Some(message) = event.pointer("/error/message").and_then(Value::as_str) {
        return Err(format!("API error: \"{message}\""));
    }

    let content = event
        .pointer("/choices/0/delta/content")
        .or_else(|| event.pointer("/choices/0/message/content"))
        .and_then(Value::as_str);

    if let Some(content) = content {
        wait_indicator.mark_first_content();
        code.push_str(content);
        preview.push_str(content)?;
    }

    Ok(false)
}

fn main() {
    let Cli {
        prompt,
        force,
        model,
        reasoning_effort,
    } = Cli::parse();
    let prompt = build_prompt(&prompt);
    let config = Config::new(
        model,
        reasoning_effort.map(|effort| effort.as_str().to_string()),
    );

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set
        .find_syntax_by_extension("sh")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let Some(theme) = theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())
    else {
        eprintln!("{}", "Failed to load syntax highlighting theme.".red());
        std::process::exit(1);
    };
    let mut preview = ShellStreamPreview::new(&syntax_set, syntax, theme);

    println!(
        "{}",
        format!(
            "Generating with {} / {}...",
            config.model, config.reasoning_effort
        )
        .bright_black()
    );
    let mut wait_indicator = WaitIndicator::start(&config.model, &config.reasoning_effort);
    let code_result = generate_code(&prompt, &config, &mut preview, &wait_indicator);
    wait_indicator.finish();
    if let Err(e) = preview.finish() {
        eprintln!("{}", e.red());
        std::process::exit(1);
    }

    match code_result {
        Ok(code) => {
            println!("{}", "Got some code!".green());

            let should_run = if force {
                true
            } else {
                Question::new(
                    ">> Run the generated program? [Y/n]"
                        .bright_black()
                        .to_string()
                        .as_str(),
                )
                .yes_no()
                .until_acceptable()
                .default(Answer::YES)
                .ask()
                .expect("Couldn't ask question.")
                    == Answer::YES
            };

            if should_run {
                config.write_to_history(code.as_str());
                let mut spinner = Spinner::new(Spinners::BouncingBar, "Executing...".into());

                let output = Command::new("bash")
                    .arg("-c")
                    .arg(code.as_str())
                    .output()
                    .unwrap_or_else(|_| {
                        spinner.stop_and_persist(
                            "✖".red().to_string().as_str(),
                            "Failed to execute the generated program.".red().to_string(),
                        );
                        std::process::exit(1);
                    });

                if !output.status.success() {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    println!("{error_output}");

                    spinner.stop_and_persist(
                        "✖".red().to_string().as_str(),
                        "The program failed.".red().to_string(),
                    );
                    std::process::exit(1);
                }

                spinner.stop_and_persist(
                    "✔".green().to_string().as_str(),
                    "Command ran successfully".green().to_string(),
                );

                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
        Err(e) => {
            eprintln!("{}", e.red());
            std::process::exit(1);
        }
    }
}

fn build_prompt(prompt: &str) -> String {
    let os_hint = if cfg!(target_os = "macos") {
        " (on macOS)"
    } else if cfg!(target_os = "linux") {
        " (on Linux)"
    } else {
        ""
    };

    let env_vars = get_env_vars();
    let env_vars_str = env_vars.join("\n");

    format!(
        "Generate a bash script for the user's task{os_hint}.\n\n\
        The following environment variables are available:\n\
        {env_vars_str}\n\n\
        Please provide only the bash script, without any additional explanations or markdown formatting.\n\n\
        Task:\n\
        {prompt}"
    )
}

fn get_env_vars() -> Vec<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("env")
        .output()
        .expect("Failed to execute command");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            line.split('=')
                .next()
                .map(|var_name| var_name.trim().to_string())
        })
        .collect()
}
