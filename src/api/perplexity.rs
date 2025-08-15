use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn query_perplexity(prompt: &str) -> Result<String, String> {
    let api_key = std::env::var("PERPLEXITY_API_KEY")
        .map_err(|_| "PERPLEXITY_API_KEY environment variable is not set")?;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let messages = json!([
        {
            "role": "system",
            "content": r#"<guidelines>
<tone>Be precise and concise</tone>
<format>Provide clear, actionable responses</format>
<constraints>
<max_length>Focus on essential information only</max_length>
<accuracy>Ensure technical accuracy</accuracy>
</constraints>
</guidelines>"#
        },
        {
            "role": "user",
            "content": prompt
        }
    ]);
    
    let request_body = json!({
        "model": "sonar",
        "messages": messages,
        "max_tokens": 1024,
        "temperature": 0.7,
        "top_p": 0.9,
        "stream": false
    });
    
    let response = client
        .post("https://api.perplexity.ai/chat/completions")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .json(&request_body)
        .send()
        .map_err(|e| format!("Failed to send request to Perplexity: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Perplexity API error: {}", response.status()));
    }
    
    let json_response: serde_json::Value = response
        .json()
        .map_err(|e| format!("Failed to parse Perplexity response: {}", e))?;
    
    // Check for API errors
    if let Some(error) = json_response.get("error") {
        return Err(format!("Perplexity API error: {}", error));
    }
    
    // Extract the response content
    let content = json_response
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .ok_or_else(|| "Invalid response structure from Perplexity API".to_string())?;
    
    Ok(content.to_string())
}