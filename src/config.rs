use colored::Colorize;
use std::{env, io::Write, process::exit};

pub struct Config {
    pub api_key: String,
    pub api_base: String,
    pub shell: String,
}

impl Config {
    pub fn new() -> Self {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
            println!("{}", "This program requires an Anthropic API key to run. Please set the ANTHROPIC_API_KEY environment variable.".red());
            exit(1);
        });
        let api_base = env::var("ANTHROPIC_API_BASE").unwrap_or_else(|_| String::from("https://api.anthropic.com"));
        let shell = env::var("SHELL").unwrap_or_else(|_| String::new());

        Self { api_key, api_base, shell }
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
