use super::Tool;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Parameters for weather queries
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WeatherParams {
    pub location: String,
    pub units: Option<TemperatureUnits>,
}

/// Temperature units
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnits {
    Celsius,
    Fahrenheit,
    Kelvin,
}

/// Weather information response
#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub location: String,
    pub temperature: f64,
    pub condition: String,
    pub humidity: f64,
    pub units: String,
}

/// A mock weather tool for demonstration
#[derive(Debug)]
pub struct WeatherTool;

impl Default for WeatherTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for WeatherTool {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn description(&self) -> &'static str {
        "Get current weather information for a location (mock implementation)"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"},
                "units": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit", "kelvin"]
                }
            },
            "required": ["location"]
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
        Box::pin(async move {
            let params: WeatherParams = serde_json::from_value(parameters).map_err(|e| {
                crate::AgentError::ToolExecution(format!("Invalid parameters: {}", e))
            })?;

            // Mock weather data - in a real implementation, you'd call a weather API
            let temperature = match params.units.clone().unwrap_or(TemperatureUnits::Celsius) {
                TemperatureUnits::Celsius => 22.5,
                TemperatureUnits::Fahrenheit => 72.5,
                TemperatureUnits::Kelvin => 295.65,
            };

            let weather_info = WeatherInfo {
                location: params.location,
                temperature,
                condition: "Partly cloudy".to_string(),
                humidity: 65.0,
                units: match params.units.unwrap_or(TemperatureUnits::Celsius) {
                    TemperatureUnits::Celsius => "°C".to_string(),
                    TemperatureUnits::Fahrenheit => "°F".to_string(),
                    TemperatureUnits::Kelvin => "K".to_string(),
                },
            };

            serde_json::to_value(weather_info).map_err(|e| {
                crate::AgentError::ToolExecution(format!("Failed to serialize result: {}", e))
            })
        })
    }
}
