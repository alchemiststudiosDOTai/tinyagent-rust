mod completion_schema;
mod schema_extraction;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Defines the `tool!` macro for declaring tools.
/// Generates a `Tool` impl with JSON Schema from `params`
/// and wires an async closure as the executor.
#[proc_macro]
pub fn tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ToolDefinition);

    let name = input.name;
    let description = input.description;
    let params_type = input.params_type;
    let execute_body = input.execute_body;

    // Convert snake_case to PascalCase for struct name
    // tbd if needed long term idk
    let struct_name = name
        .value()
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>();

    let tool_struct = quote::format_ident!("{}", struct_name);

    let expanded = quote! {
        #[derive(Debug)]
        pub struct #tool_struct;

        impl tiny_agent_rs::tools::Tool for #tool_struct {
            fn name(&self) -> &'static str {
                #name
            }

            fn description(&self) -> &'static str {
                #description
            }

            fn parameters_schema(&self) -> serde_json::Value {
                let schema = schemars::schema_for!(#params_type);
                serde_json::to_value(&schema.schema).unwrap_or_else(|_| {
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    })
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
                    let params: #params_type = serde_json::from_value(parameters)
                        .map_err(|e| tiny_agent_rs::AgentError::ToolExecution(
                            format!("Invalid parameters for {}: {}", #name, e)
                        ))?;

                    let handler = #execute_body;
                    handler(params)
                        .await
                        .map_err(|e| tiny_agent_rs::AgentError::ToolExecution(e))
                })
            }
        }
    };

    TokenStream::from(expanded)
}

struct ToolDefinition {
    name: syn::LitStr,
    description: syn::LitStr,
    params_type: syn::Type,
    execute_body: syn::ExprClosure,
}

fn parse_named_assignment<T: syn::parse::Parse>(
    input: syn::parse::ParseStream,
    keyword: &str,
) -> syn::Result<T> {
    let ident: syn::Ident = input.parse()?;
    if ident != keyword {
        return Err(syn::Error::new_spanned(
            ident,
            format!("expected '{keyword}'"),
        ));
    }
    input.parse::<syn::Token![=]>()?;
    let value = input.parse::<T>()?;
    input.parse::<syn::Token![,]>()?;
    Ok(value)
}

impl syn::parse::Parse for ToolDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = parse_named_assignment::<syn::LitStr>(input, "name")?;
        let description = parse_named_assignment::<syn::LitStr>(input, "description")?;
        let params_type = parse_named_assignment::<syn::Type>(input, "params")?;
        let execute_body: syn::ExprClosure = input.parse()?;

        Ok(ToolDefinition {
            name,
            description,
            params_type,
            execute_body,
        })
    }
}

#[proc_macro_attribute]
pub fn completion_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    completion_schema::completion_schema(attr, item)
}
