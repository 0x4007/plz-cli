use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

use crate::config::Config;
use crate::utils::logging::log_to_file;
use crate::utils::markdown::extract_code_from_markdown;
use super::error::ApiError;

pub fn generate_code(prompt: &str, config: &Config) -> Result<String, ApiError> {
    if std::env::var("PLZ_DEBUG").is_ok() {
        log_debug_info(prompt, config);
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| ApiError::ClientBuild(e.to_string()))?;

    let response = client
        .post(&config.api_base)
        .json(&json!({
            "model": &config.model,
            "temperature": 1e-10,
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
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("HTTP-Referer", "https://github.com/0x4007/plz-cli")
        .header("Content-Type", "application/json")
        .send()
        .map_err(|e| ApiError::Request(e.to_string()))?;

    let status = response.status();
    if status.is_client_error() {
        let error_json = response
            .json::<serde_json::Value>()
            .map_err(|e| ApiError::Parse(e.to_string()))?;
        let error_message = error_json["error"]["message"]
            .as_str()
            .unwrap_or("Unknown error");
        return Err(ApiError::Client(error_message.to_string()));
    } else if status.is_server_error() {
        return Err(ApiError::Server(status.as_u16()));
    }

    let response_json = response
        .json::<serde_json::Value>()
        .map_err(|e| ApiError::Parse(e.to_string()))?;

    let content = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .ok_or_else(|| ApiError::InvalidResponse)?;

    Ok(extract_code_from_markdown(content))
}

fn log_debug_info(prompt: &str, config: &Config) {
    let log_content = format!("<system>\n{}\n</system>\n\n<user>\n{}\n</user>", 
        config.system_prompt.trim(),
        prompt.trim()
    );
    
    // Determine the step based on the prompt content
    let step = if prompt.contains("Perplexity search results suggest:") {
        "step3_retry"
    } else {
        "step1_initial"
    };
    
    log_to_file(&log_content, "openrouter_request", step);
}