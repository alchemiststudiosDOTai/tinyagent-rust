use std::time::Duration;

use reqwest::StatusCode;
use serde_json::{json, Value};

use crate::error::{AgentError, Result};

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";
const MAX_RETRIES: usize = 3;

#[derive(Clone, Debug)]
pub struct OpenAIClient {
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self.base_url = base_url.into();
    }

    pub async fn chat_completion(&self, body: &Value, timeout: Duration) -> Result<Value> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| AgentError::Unknown(format!("Failed to build HTTP client: {err}")))?;

        let mut attempt = 0;
        let mut backoff = Duration::from_millis(250);

        loop {
            let request_url = build_chat_url(&self.base_url);

            let response = client
                .post(&request_url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header(
                    "HTTP-Referer",
                    "https://github.com/tunahorse/tinyagent-rust",
                )
                .header("X-Title", "tiny-agent-rs")
                .json(body)
                .send()
                .await
                .map_err(|err| AgentError::Unknown(format!("HTTP request failed: {err}")))?;

            let status = response.status();
            let headers = response.headers().clone();
            let response_text = response
                .text()
                .await
                .map_err(|err| AgentError::Unknown(format!("Failed to read response: {err}")))?;

            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after_duration = headers
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or(backoff);

                if attempt < MAX_RETRIES {
                    tokio::time::sleep(retry_after_duration).await;
                    attempt += 1;
                    backoff *= 2;
                    continue;
                }

                return Err(AgentError::RateLimit {
                    retry_after: retry_after_duration.as_secs().max(1),
                });
            }

            if status.is_server_error() && attempt < MAX_RETRIES {
                tokio::time::sleep(backoff).await;
                attempt += 1;
                backoff *= 2;
                continue;
            }

            let response_json: Value = serde_json::from_str(&response_text)
                .map_err(|err| AgentError::Unknown(format!("Failed to parse JSON: {err}")))?;

            if !status.is_success() {
                let api_message = response_json
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(|value| value.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(response_text.clone());

                return Err(AgentError::Unknown(format!(
                    "HTTP {} error: {}",
                    status, api_message
                )));
            }

            if let Some(error) = response_json.get("error") {
                let error_message = error
                    .get("message")
                    .and_then(|value| value.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| error.to_string());
                return Err(AgentError::Unknown(format!("API error: {}", error_message)));
            }

            return Ok(response_json);
        }
    }
}

fn build_chat_url(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else {
        format!("{}/chat/completions", trimmed)
    }
}

#[derive(Clone, Debug)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<Value>,
    tools: Vec<Value>,
    tool_choice: Option<Value>,
    max_tokens: Option<u32>,
    response_format: Option<Value>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Value>) -> Self {
        Self {
            model: model.into(),
            messages,
            tools: Vec::new(),
            tool_choice: None,
            max_tokens: None,
            response_format: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<Value>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_tool_choice(mut self, tool_choice: Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    #[allow(dead_code)]
    pub fn with_response_format(mut self, response_format: Value) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn into_value(self) -> Value {
        let mut body = json!({
            "model": self.model,
            "messages": self.messages,
        });

        if !self.tools.is_empty() {
            body["tools"] = Value::Array(self.tools);
        }

        if let Some(tool_choice) = self.tool_choice {
            body["tool_choice"] = tool_choice;
        }

        if let Some(max_tokens) = self.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }

        if let Some(response_format) = self.response_format {
            body["response_format"] = response_format;
        }

        body
    }
}
