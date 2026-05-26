use colored::Colorize;
use std::{env, io::Write, process::exit};

const UOS_AI_CHAT_COMPLETIONS_URL: &str = "https://ai.ubq.fi/v1/chat/completions";
const PROMPT_CACHE_KEY_PREFIX: &str = "plz-cli:bash-script:v1";
const DEFAULT_MODEL: &str = "gpt-5.3-codex-spark";
const DEFAULT_REASONING_EFFORT: &str = "xhigh";
const DEFAULT_SYSTEM_PROMPT: &str =
    "You are a helpful assistant that generates bash scripts based on user prompts.";

pub struct Config {
    pub api_token: String,
    pub shell: String,
    pub model: String,
    pub reasoning_effort: String,
    pub prompt_cache_key: String,
    pub chat_completions_url: String,
    pub system_prompt: String,
}

impl Config {
    pub fn new(model: Option<String>, reasoning_effort: Option<String>) -> Self {
        let api_token = env::var("UOS_AI_TOKEN").or_else(|_| env::var("DENO_DEPLOY_TOKEN")).unwrap_or_else(|_| {
            println!("{}", "This program requires a UOS AI Gateway token to run. Please set UOS_AI_TOKEN or DENO_DEPLOY_TOKEN.".red());
            exit(1);
        });

        let shell = env::var("SHELL").unwrap_or_else(|_| String::new());

        let model = model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let reasoning_effort =
            reasoning_effort.unwrap_or_else(|| DEFAULT_REASONING_EFFORT.to_string());
        let prompt_cache_key = prompt_cache_key(&model, &reasoning_effort);

        Self {
            api_token,
            shell,
            model,
            reasoning_effort,
            prompt_cache_key,
            chat_completions_url: UOS_AI_CHAT_COMPLETIONS_URL.to_string(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
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
            eprintln!("Failed to write to history file: {err}");
            exit(1);
        }
    }
}

fn prompt_cache_key(model: &str, reasoning_effort: &str) -> String {
    let model = sanitize_cache_key_part(model);
    let reasoning_effort = sanitize_cache_key_part(reasoning_effort);
    format!("{PROMPT_CACHE_KEY_PREFIX}:{model}:{reasoning_effort}")
}

fn sanitize_cache_key_part(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
