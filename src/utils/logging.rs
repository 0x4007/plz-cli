use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global session ID that persists for the entire run
static SESSION_ID: Lazy<Mutex<String>> = Lazy::new(|| {
    let session_id = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    Mutex::new(session_id)
});

pub fn get_session_id() -> String {
    SESSION_ID.lock().unwrap().clone()
}

pub fn log_to_file(content: &str, log_type: &str, step: &str) {
    if std::env::var("PLZ_DEBUG").is_ok() {
        let session_id = get_session_id();
        let log_path = format!("logs/{}_{}_{}.log", session_id, step, log_type);
        
        if let Err(e) = std::fs::create_dir_all("logs") {
            eprintln!("Failed to create logs directory: {}", e);
            return;
        }
        
        if let Err(e) = std::fs::write(&log_path, content) {
            eprintln!("Failed to write to log file: {}", e);
        } else {
            eprintln!("Logged to: {}", log_path);
        }
    }
}