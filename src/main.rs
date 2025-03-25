#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use bat::PrettyPrinter;
use clap::Parser;
use colored::Colorize;
use config::Config;
use question::{Answer, Question};
use reqwest::blocking::Client;
use serde_json::json;
use spinners::{Spinner, Spinners};
use std::process::Command;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Description of the command to execute
    prompt: Vec<String>,

    /// Run the generated program without asking for confirmation
    #[clap(short = 'y', long)]
    force: bool,

    /// Remove token limit for complex scripts (may increase API costs)
    #[clap(long)]
    extended: bool,
}

fn main() {
    let cli = Cli::parse();
    let config = Config::new();

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap();
    let mut spinner = Spinner::new(Spinners::BouncingBar, format!(
        "Generating your command{} (this may take a while)...",
        if cli.extended { " in extended mode" } else { "" }
    ).into());
    let api_addr = "https://openrouter.ai/api/v1/chat/completions".to_string();
    let response = client
        .post(api_addr)
        .json(&json!({
            "model": "anthropic/claude-3-7-sonnet",
            "max_tokens": if cli.extended { 32000 } else { 1000 },
            "temperature": 0,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant that generates bash scripts based on user prompts."
                },
                {
                    "role": "user",
                    "content": build_prompt(&cli.prompt.join(" "))
                }
            ]
        }))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", "https://github.com/0x4007/plz-cli")
        .header("Content-Type", "application/json")
        .send()
        .unwrap_or_else(|e| {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("API request failed: {}", e).red().to_string(),
            );
            std::process::exit(1)
        });

    let status_code = response.status();
    if status_code.is_client_error() {
        let response_body = response.json::<serde_json::Value>().unwrap_or_else(|e| {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("Failed to parse error response: {}", e).red().to_string(),
            );
            std::process::exit(1)
        });
        let error_message = response_body["error"]["message"].as_str().unwrap_or("Unknown error");
        spinner.stop_and_persist(
            "✖".red().to_string().as_str(),
            format!("API error: \"{error_message}\"").red().to_string(),
        );
        std::process::exit(1);
    } else if status_code.is_server_error() {
        spinner.stop_and_persist(
            "✖".red().to_string().as_str(),
            format!("OpenRouter is currently experiencing problems. Status code: {status_code}")
                .red()
                .to_string(),
        );
        std::process::exit(1);
    }

    let code = response.json::<serde_json::Value>()
        .unwrap_or_else(|e| {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                format!("Failed to parse API response: {}", e).red().to_string(),
            );
            std::process::exit(1)
        })
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_else(|| {
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                "Invalid API response format".red().to_string(),
            );
            std::process::exit(1)
        })
        .trim()
        .to_string();

    spinner.stop_and_persist(
        "✔".green().to_string().as_str(),
        "Got some code!".green().to_string(),
    );

    PrettyPrinter::new()
        .input_from_bytes(code.as_bytes())
        .language("bash")
        .grid(true)
        .print()
        .unwrap();

    let should_run = if cli.force {
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
        spinner = Spinner::new(Spinners::BouncingBar, "Executing...".into());

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
            spinner.stop_and_persist(
                "✖".red().to_string().as_str(),
                "The program threw an error.".red().to_string(),
            );
            println!("{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }

        spinner.stop_and_persist(
            "✔".green().to_string().as_str(),
            "Command ran successfully".green().to_string(),
        );

        println!("{}", String::from_utf8_lossy(&output.stdout));
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
        "Generate a bash script for the following task{os_hint}: {prompt}\n\n\
        The following environment variables are available:\n\
        {env_vars_str}\n\n\
        Please provide only the bash script, without any additional explanations or markdown formatting."
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
