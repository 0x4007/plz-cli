use colored::Colorize;
use std::{env, io::Write, process::exit};

pub struct Config {
    pub api_key: String,
    pub shell: String,
    pub model: String,
    pub api_base: String,
    pub system_prompt: String,
}

impl Config {
    pub fn new() -> Self {
        let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
            println!("{}", "This program requires an OpenRouter API key to run. Please set the OPENROUTER_API_KEY environment variable.".red());
            exit(1);
        });

        let shell = env::var("SHELL").unwrap_or_else(|_| String::new());

        // Model configuration with default to Claude 3.7 Sonnet
        let model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-opus-4".to_string());

        // API base URL configuration with default
        let api_base = env::var("OPENROUTER_API_BASE")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());

        // System prompt configuration with default
        let system_prompt = env::var("OPENROUTER_SYSTEM_PROMPT")
            .unwrap_or_else(|_| "You are a helpful assistant that generates bash scripts based on user prompts.".to_string());

        Self {
            api_key,
            shell,
            model,
            api_base,
            system_prompt,
        }
    }

    pub fn write_to_history(&self, code: &str) {
        let history_file = match self.shell.as_str() {
            "/bin/bash" => std::env::var("HOME").unwrap() + "/.bash_history",
            "/bin/zsh" => std::env::var("HOME").unwrap() + "/.zsh_history",
            _ => return,
        };

        if let Err(err) = std::fs::OpenOptions::new()
            .append(true)
            .open(history_file)
            .and_then(|mut file| file.write_all(format!("{code}\n").as_bytes()))
        {
            eprintln!("Failed to write to history file: {}", err);
            exit(1);
        }
    }
}
