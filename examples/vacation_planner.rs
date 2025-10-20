use serde::Deserialize;
use serde_json::json;
use tiny_agent_rs::{
    tools::{JinaReaderTool, Tool},
    vacation_types::VacationPlan,
    Agent, FunctionFactory,
};

#[derive(Debug)]
struct BudgetCalculator;

#[derive(Debug, Deserialize)]
struct BudgetParams {
    nights: u32,
    nightly_rate: f64,
    #[serde(default)]
    travelers: Option<u32>,
}

impl Tool for BudgetCalculator {
    fn name(&self) -> &'static str {
        "budget_calculator"
    }

    fn description(&self) -> &'static str {
        "Estimate lodging budget given nights, nightly_rate, and optional traveler count"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "nights": {
                    "type": "integer",
                    "description": "Number of nights the trip will last"
                },
                "nightly_rate": {
                    "type": "number",
                    "description": "Estimated nightly rate in USD"
                },
                "travelers": {
                    "type": "integer",
                    "description": "Number of travelers splitting the cost"
                }
            },
            "required": ["nights", "nightly_rate"],
            "additionalProperties": false
        })
    }

    fn execute(
        &self,
        parameters: serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<serde_json::Value, tiny_agent_rs::AgentError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let params: BudgetParams = serde_json::from_value(parameters).map_err(|err| {
                tiny_agent_rs::AgentError::ToolExecution(format!(
                    "Invalid budget parameters: {}",
                    err
                ))
            })?;

            let total = params.nightly_rate * params.nights as f64;
            let per_person = params
                .travelers
                .filter(|&t| t > 0)
                .map(|t| total / t as f64);

            Ok(json!({
                "nights": params.nights,
                "nightly_rate": params.nightly_rate,
                "total_cost": total,
                "travelers": params.travelers,
                "per_person": per_person
            }))
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::try_init().ok();
    // Load credentials
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let jina_key = std::env::var("JINA_API_KEY")?;

    let mut factory = FunctionFactory::new();
    factory.register_tool(JinaReaderTool::new(jina_key));
    factory.register_tool(BudgetCalculator);

    let model = std::env::var("MODEL").unwrap_or_else(|_| "openai/gpt-4.1".to_string());
    let agent = Agent::new(api_key, factory)
        .with_max_iterations(12)
        .with_model(model)
        .with_completion_schema::<VacationPlan>();

    println!("=== Vacation Planner Agent ===\n");

    let task = r#"
    Plan a long-weekend trip to Paris for two adults.
    - Use the jina_reader tool to pull highlights from https://en.parisinfo.com/discover-paris/paris-in-1-2-or-3-days-i126
    - Estimate the hotel cost assuming 3 nights at $240/night split between the travelers.
    - Present a concise itinerary with budget notes.
    - Return the final answer using the structured vacation schema.
    "#;

    println!("Task:{}\n", task);

    let result = agent.run_with_steps(task).await?;

    println!("{}", result.replay());

    println!("\n--- Detailed Explanation ---");
    println!("{}", result.explain());

    if result.has_structured() {
        let plan: VacationPlan = result
            .deserialize_structured()
            .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;

        println!("\n--- Vacation Plan Summary ---");
        println!(
            "Destination: {} ({} nights, {} travelers)",
            plan.destination,
            plan.nights,
            plan.travelers.unwrap_or(2)
        );
        if let Some(currency) = &plan.currency {
            println!(
                "Total Budget: {:.2} {} (per person: {:.2})",
                plan.total_budget,
                currency,
                plan.budget_per_person.unwrap_or(plan.total_budget)
            );
        } else {
            println!(
                "Total Budget: {:.2} (per person: {:.2})",
                plan.total_budget,
                plan.budget_per_person.unwrap_or(plan.total_budget)
            );
        }

        println!("\nItinerary:");
        for day in &plan.itinerary {
            let title = day.title.as_deref().unwrap_or("Daily Highlights");
            println!(
                "  Day {} - {} (est. {:.2})",
                day.day, title, day.estimated_cost
            );
            for activity in &day.activities {
                println!("    • {}", activity);
            }
            if let Some(notes) = &day.notes {
                println!("    Notes: {}", notes);
            }
        }

        println!(
            "\nStructured JSON:\n{}",
            serde_json::to_string_pretty(&plan)?
        );
    } else {
        eprintln!("\nStructured payload was not returned by the agent.");
    }

    Ok(())
}
