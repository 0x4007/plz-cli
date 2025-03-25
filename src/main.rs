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
}

fn generate_code(prompt: &str, max_tokens: u32, config: &Config) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build client: {}", e))?;

    let api_addr = "https://openrouter.ai/api/v1/chat/completions".to_string();
    let response = client
        .post(api_addr)
        .json(&json!({
            "model": "anthropic/claude-3-7-sonnet",
            "max_tokens": max_tokens,
            "temperature": 0,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a helpful assistant that generates bash scripts based on user prompts."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", "https://github.com/0x4007/plz-cli")
        .header("Content-Type", "application/json")
        .send()
        .map_err(|e| format!("API request failed: {}", e))?;

    let status = response.status();
    if status.is_client_error() {
        let error_json = response
            .json::<serde_json::Value>()
            .map_err(|e| format!("Failed to parse error response: {}", e))?;
        let error_message = error_json["error"]["message"]
            .as_str()
            .unwrap_or("Unknown error");
        return Err(format!("API error: \"{}\"", error_message));
    } else if status.is_server_error() {
        return Err(format!(
            "OpenRouter is currently experiencing problems. Status code: {}",
            status
        ));
    }

    let response_json = response
        .json::<serde_json::Value>()
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    let content = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .ok_or_else(|| "Invalid API response format".to_string())?;

    Ok(content.trim().to_string())
}

fn main() {
    let cli = Cli::parse();
    let config = Config::new();
    let mut current_token_limit = 1000;
    let prompt = build_prompt(&cli.prompt.join(" "));
    let mut spinner;

    loop {
        spinner = Spinner::new(Spinners::BouncingBar, format!(
            "Generating your command with {} tokens...",
            current_token_limit
        ).into());

        let code_result = generate_code(&prompt, current_token_limit, &config);

        match code_result {
            Ok(code) => {
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
                        let error_output = String::from_utf8_lossy(&output.stderr);
                        println!("{}", error_output);

                        if current_token_limit >= 64000 {
                            spinner.stop_and_persist(
                                "✖".red().to_string().as_str(),
                                "The program failed even with maximum token limit.".red().to_string(),
                            );
                            std::process::exit(1);
                        }

                        current_token_limit *= 2;
                        continue;
                    }

                    spinner.stop_and_persist(
                        "✔".green().to_string().as_str(),
                        "Command ran successfully".green().to_string(),
                    );

                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
                break;
            }
            Err(e) => {
                spinner.stop_and_persist(
                    "✖".red().to_string().as_str(),
                    e.red().to_string(),
                );
                if current_token_limit >= 64000 {
                    spinner.stop_and_persist(
                        "✖".red().to_string().as_str(),
                        "Failed to generate code even with maximum token limit.".red().to_string(),
                    );
                    std::process::exit(1);
                }

                current_token_limit *= 2;
                continue;
            }
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
