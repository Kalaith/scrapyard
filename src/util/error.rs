//! Custom error types for graceful error handling

use thiserror::Error;

/// Game-specific error types
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {message}")]
    Config { message: String },
}

/// Convenience Result type using GameError
pub type Result<T> = std::result::Result<T, GameError>;
