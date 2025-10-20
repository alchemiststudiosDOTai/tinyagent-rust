use crate::tools::FunctionFactory;

/// Generate a planning prompt for the agent before tool execution
pub fn generate_planning_prompt(
    task: &str,
    available_tools: &[String],
    iteration: usize,
) -> String {
    let tools_list = if available_tools.is_empty() {
        "No tools available".to_string()
    } else {
        available_tools.join(", ")
    };

    if iteration == 1 {
        format!(
            "Task: {}\n\nAvailable tools: {}\n\nThink step-by-step about how to solve this task. What tools will you need? What's your approach?",
            task, tools_list
        )
    } else {
        format!(
            "Continuing task: {}\n\nIteration {}\nAvailable tools: {}\n\nBased on previous observations, what should be the next step?",
            task, iteration, tools_list
        )
    }
}

/// Extract tool names from FunctionFactory for planning context
pub fn get_tool_names(factory: &FunctionFactory) -> Vec<String> {
    factory
        .get_openai_tools()
        .iter()
        .filter_map(|tool| {
            tool.get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
        .collect()
}

/// Generate a simplified planning prompt that encourages direct tool use
pub fn generate_tool_planning_prompt(task: &str, factory: &FunctionFactory) -> String {
    let tool_descriptions: Vec<String> = factory
        .get_openai_tools()
        .iter()
        .filter_map(|tool| {
            let function = tool.get("function")?;
            let name = function.get("name")?.as_str()?;
            let description = function.get("description")?.as_str()?;
            Some(format!("- {}: {}", name, description))
        })
        .collect();

    if tool_descriptions.is_empty() {
        format!("Task: {}\n\nHow would you approach solving this?", task)
    } else {
        format!(
            "Task: {}\n\nAvailable tools:\n{}\n\nWhat's your plan to solve this task?",
            task,
            tool_descriptions.join("\n")
        )
    }
}

/// Check if a response indicates the agent is planning vs. executing
pub fn is_planning_response(content: &str) -> bool {
    let planning_indicators = [
        "plan", "approach", "strategy", "steps:", "first,", "then,", "finally,", "need to",
        "should", "will use",
    ];

    let content_lower = content.to_lowercase();
    planning_indicators
        .iter()
        .any(|indicator| content_lower.contains(indicator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_planning_prompt() {
        let tools = vec!["search".to_string(), "calculator".to_string()];
        let prompt = generate_planning_prompt("Calculate 2+2", &tools, 1);

        assert!(prompt.contains("Calculate 2+2"));
        assert!(prompt.contains("search, calculator"));
        assert!(prompt.contains("Think step-by-step"));
    }

    #[test]
    fn test_generate_planning_prompt_continuation() {
        let tools = vec!["search".to_string()];
        let prompt = generate_planning_prompt("Find info", &tools, 3);

        assert!(prompt.contains("Iteration 3"));
        assert!(prompt.contains("next step"));
    }

    #[test]
    fn test_empty_tools() {
        let tools: Vec<String> = vec![];
        let prompt = generate_planning_prompt("Task", &tools, 1);

        assert!(prompt.contains("No tools available"));
    }

    #[test]
    fn test_is_planning_response() {
        assert!(is_planning_response("My plan is to use the search tool"));
        assert!(is_planning_response(
            "First, I will search. Then, I will analyze."
        ));
        assert!(is_planning_response("I need to calculate the result"));
        assert!(!is_planning_response("The result is 42"));
        assert!(!is_planning_response("Done"));
    }

    #[test]
    fn test_tool_planning_prompt_empty() {
        let factory = FunctionFactory::new();
        let prompt = generate_tool_planning_prompt("Test task", &factory);

        assert!(prompt.contains("Test task"));
        assert!(prompt.contains("How would you approach"));
    }
}
