#[derive(Debug)]
pub enum ApiError {
    ClientBuild(String),
    Request(String),
    Client(String),
    Server(u16),
    Parse(String),
    InvalidResponse,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::ClientBuild(e) => write!(f, "Failed to build client: {}", e),
            ApiError::Request(e) => write!(f, "API request failed: {}", e),
            ApiError::Client(e) => write!(f, "API error: \"{}\"", e),
            ApiError::Server(code) => write!(f, "OpenRouter is currently experiencing problems. Status code: {}", code),
            ApiError::Parse(e) => write!(f, "Failed to parse API response: {}", e),
            ApiError::InvalidResponse => write!(f, "Invalid API response format"),
        }
    }
}

impl std::error::Error for ApiError {}