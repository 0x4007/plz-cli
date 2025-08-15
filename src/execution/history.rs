use std::fmt;

#[derive(Debug, Clone)]
pub struct AttemptHistory {
    pub attempt_number: u8,
    pub generated_code: String,
    pub error_output: String,
    pub perplexity_analysis: Option<String>,
}

impl fmt::Display for AttemptHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<attempt number=\"{}\">\n", self.attempt_number)?;
        write!(f, "<openrouter_generated_code>\n{}\n</openrouter_generated_code>\n", self.generated_code.trim())?;
        write!(f, "<bash_error>\n{}\n</bash_error>\n", self.error_output.trim())?;
        if let Some(analysis) = &self.perplexity_analysis {
            write!(f, "<perplexity_response>\n{}\n</perplexity_response>\n", analysis.trim())?;
        }
        write!(f, "</attempt>")
    }
}

pub fn format_attempt_history(history: &[AttemptHistory]) -> String {
    if history.is_empty() {
        return String::new();
    }
    
    let mut result = String::from("<previous_attempts>\n");
    for attempt in history {
        result.push_str(&format!("{}\n\n", attempt));
    }
    result.push_str("</previous_attempts>");
    result
}