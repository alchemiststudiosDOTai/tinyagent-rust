use super::Tool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Parameters accepted by the Jina reader tool
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JinaReaderParams {
    /// Fully-qualified URL to fetch (e.g. <https://www.example.com>)
    pub url: String,
    /// When true, bypass cached snapshot
    #[serde(default)]
    pub no_cache: Option<bool>,
}

/// Response returned by Jina reader
#[derive(Debug, Serialize, Deserialize)]
pub struct JinaReaderResponse {
    pub title: Option<String>,
    pub url_source: Option<String>,
    pub published_time: Option<String>,
    pub markdown: Option<String>,
    pub raw: String,
}

/// Tool that calls the Jina reader API and returns markdown content
#[derive(Debug, Clone)]
pub struct JinaReaderTool {
    api_key: String,
    client: Client,
}

impl JinaReaderTool {
    /// Create a new tool using the provided API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    /// Build the tool using the `JINA_API_KEY` environment variable
    pub fn from_env() -> Result<Self, crate::AgentError> {
        let api_key = std::env::var("JINA_API_KEY")
            .map_err(|_| crate::AgentError::Config("Missing JINA_API_KEY env var".to_string()))?;
        Ok(Self::new(api_key))
    }
}

impl Tool for JinaReaderTool {
    fn name(&self) -> &'static str {
        "jina_reader"
    }

    fn description(&self) -> &'static str {
        "Fetch markdown content for a URL using the Jina reader API"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Fully qualified URL to fetch"
                },
                "no_cache": {
                    "type": "boolean",
                    "description": "Set true to bypass cached snapshot"
                }
            },
            "required": ["url"]
        })
    }

    fn execute(
        &self,
        parameters: serde_json::Value,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, crate::AgentError>>
                + Send
                + '_,
        >,
    > {
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        Box::pin(async move {
            let params: JinaReaderParams = serde_json::from_value(parameters).map_err(|err| {
                crate::AgentError::ToolExecution(format!("Invalid parameters: {}", err))
            })?;

            let target_url = if params.url.starts_with("https://r.jina.ai/") {
                params.url
            } else {
                format!("https://r.jina.ai/{}", params.url)
            };

            let mut request = client
                .get(&target_url)
                .header("Authorization", format!("Bearer {}", api_key));

            if params.no_cache.unwrap_or(false) {
                request = request.header("Cache-Control", "no-cache");
            }

            let response = request.send().await.map_err(|err| {
                crate::AgentError::ToolExecution(format!("Failed to call Jina reader: {}", err))
            })?;

            if !response.status().is_success() {
                return Err(crate::AgentError::ToolExecution(format!(
                    "Jina reader returned status {}",
                    response.status()
                )));
            }

            let body = response.text().await.map_err(|err| {
                crate::AgentError::ToolExecution(format!("Failed to read Jina response: {}", err))
            })?;

            let parsed = parse_jina_response(&body);

            serde_json::to_value(parsed).map_err(|err| {
                crate::AgentError::ToolExecution(format!("Failed to serialize response: {}", err))
            })
        })
    }
}

fn parse_jina_response(raw: &str) -> JinaReaderResponse {
    let mut title = None;
    let mut url_source = None;
    let mut published_time = None;
    let mut markdown_lines: Vec<String> = Vec::new();
    let mut in_markdown = false;

    for line in raw.lines() {
        if let Some(value) = line.strip_prefix("Title: ") {
            title = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = line.strip_prefix("URL Source: ") {
            url_source = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = line.strip_prefix("Published Time: ") {
            published_time = Some(value.trim().to_string());
            continue;
        }

        if let Some(value) = line.strip_prefix("Markdown Content:") {
            in_markdown = true;
            let trimmed = value.trim_start();
            if !trimmed.is_empty() {
                markdown_lines.push(trimmed.to_string());
            }
            continue;
        }

        if in_markdown {
            markdown_lines.push(line.to_string());
        }
    }

    JinaReaderResponse {
        title,
        url_source,
        published_time,
        markdown: if markdown_lines.is_empty() {
            None
        } else {
            Some(markdown_lines.join("\n"))
        },
        raw: raw.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_jina_response() {
        let raw =
            "Title: Example\nURL Source: https://example.com\nMarkdown Content:\nLine 1\nLine 2";
        let parsed = parse_jina_response(raw);
        assert_eq!(parsed.title.as_deref(), Some("Example"));
        assert_eq!(parsed.url_source.as_deref(), Some("https://example.com"));
        assert_eq!(parsed.markdown.as_deref(), Some("Line 1\nLine 2"));
        assert!(parsed.raw.contains("Markdown Content"));
    }
}
