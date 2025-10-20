use thiserror::Error;

/// Main error type for the agent system
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("OpenAI API error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid function call: {0}")]
    InvalidFunctionCall(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Maximum iterations exceeded: {0}")]
    MaxIterations(usize),

    #[error("Rate limit exceeded: retry after {retry_after}s")]
    RateLimit { retry_after: u64 },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, AgentError>;

impl AgentError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            AgentError::OpenAI(openai_err) => {
                matches!(openai_err, async_openai::error::OpenAIError::ApiError(_))
            }
            AgentError::Validation(_) => true,
            AgentError::RateLimit { .. } => true,
            AgentError::Timeout(_) => true,
            _ => false,
        }
    }

    /// Get the error code for structured responses
    pub fn error_code(&self) -> &'static str {
        match self {
            AgentError::Config(_) => "CONFIG_ERROR",
            AgentError::OpenAI(_) => "OPENAI_ERROR",
            AgentError::Serialization(_) => "SERIALIZATION_ERROR",
            AgentError::Validation(_) => "VALIDATION_ERROR",
            AgentError::ToolExecution(_) => "TOOL_EXECUTION_ERROR",
            AgentError::ToolNotFound(_) => "TOOL_NOT_FOUND",
            AgentError::InvalidFunctionCall(_) => "INVALID_FUNCTION_CALL",
            AgentError::Timeout(_) => "TIMEOUT_ERROR",
            AgentError::MaxIterations(_) => "MAX_ITERATIONS_EXCEEDED",
            AgentError::RateLimit { .. } => "RATE_LIMIT_ERROR",
            AgentError::Unknown(_) => "UNKNOWN_ERROR",
        }
    }

    /// Convert to a structured error payload
    pub fn to_error_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string(),
                "retryable": self.is_retryable()
            }
        })
    }
}
