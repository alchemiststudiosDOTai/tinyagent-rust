use crate::{
    error::{AgentError, Result},
    schemas::{CompletionSchema, SchemaHandle},
    services::openai_client::OpenAIClient,
    tools::FunctionFactory,
};
use serde_json::{json, Value};
use std::time::Duration;

/// Main agent
#[derive(Debug)]
pub struct Agent {
    openai_client: OpenAIClient,
    function_factory: FunctionFactory,
    model: String,
    max_iterations: usize,
    max_tokens: Option<u32>,
    timeout: Duration,
    completion_schema: Option<SchemaHandle>,
}

impl Agent {
    pub fn new(api_key: String, function_factory: FunctionFactory) -> Self {
        Self {
            openai_client: OpenAIClient::new(api_key),
            function_factory,
            model: "openai/gpt-4.1-mini".to_string(),
            max_iterations: 10,
            max_tokens: Some(1000),
            timeout: Duration::from_secs(120),
            completion_schema: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.openai_client.set_base_url(base_url);
        self
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_completion_schema<T: CompletionSchema>(mut self) -> Self {
        self.completion_schema = Some(T::schema().clone());
        self
    }

    pub(crate) fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    pub(crate) fn completion_schema(&self) -> Option<&SchemaHandle> {
        self.completion_schema.as_ref()
    }

    pub(crate) fn function_factory(&self) -> &FunctionFactory {
        &self.function_factory
    }

    pub(crate) fn model(&self) -> &str {
        &self.model
    }

    pub(crate) fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub(crate) fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn clear_completion_schema(mut self) -> Self {
        self.completion_schema = None;
        self
    }

    pub async fn run(&self, prompt: &str) -> Result<String> {
        let messages = vec![
            json!({
                "role": "system",
                "content": "You are a helpful assistant with access to tools. Use tools when necessary to provide accurate information. Be concise and helpful. When you are ready to give the final response, you MUST call the `final_answer` tool with an `answer` string instead of replying directly."
            }),
            json!({
                "role": "user",
                "content": prompt
            }),
        ];

        self.run_with_messages(messages).await
    }

    pub(crate) async fn make_raw_request(&self, request_body: &Value) -> Result<Value> {
        self.openai_client
            .chat_completion(request_body, self.timeout)
            .await
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            AgentError::Config(
                "OPENAI_API_KEY environment variable must be set before creating an Agent"
                    .to_string(),
            )
        })?;
        let function_factory = FunctionFactory::new();
        let mut agent = Self::new(api_key, function_factory);
        if let Ok(base_url) =
            std::env::var("OPENAI_BASE_URL").or_else(|_| std::env::var("OPENROUTER_BASE_URL"))
        {
            agent.openai_client.set_base_url(base_url);
        }
        Ok(agent)
    }
}
