use std::any::{type_name, TypeId};

use serde_json::Value;

use crate::{
    error::{AgentError, Result},
    schemas::{CompletionSchema, SchemaHandle},
};

#[derive(Clone, Debug)]
pub struct StructuredPayload {
    schema: SchemaHandle,
    value: Value,
}

impl StructuredPayload {
    pub fn new(schema: SchemaHandle, value: Value) -> Self {
        Self { schema, value }
    }

    pub fn schema(&self) -> &SchemaHandle {
        &self.schema
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn into_inner(self) -> (SchemaHandle, Value) {
        (self.schema, self.value)
    }

    pub fn deserialize<T>(&self) -> Result<T>
    where
        T: CompletionSchema,
    {
        deserialize_structured_response::<T>(&self.value, &self.schema)
    }
}

pub fn deserialize_structured_response<T>(payload: &Value, schema: &SchemaHandle) -> Result<T>
where
    T: CompletionSchema,
{
    ensure_schema_matches::<T>(schema)?;

    let raw = payload.to_string();
    let mut deserializer = serde_json::Deserializer::from_str(&raw);
    let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
        let path = err.path().to_string();
        let location = if path.is_empty() {
            "<root>".to_string()
        } else {
            path
        };
        AgentError::Validation(format!(
            "failed to deserialize `{}` at {}: {}",
            schema.schema_name(),
            location,
            err
        ))
    })?;

    Ok(value)
}

fn ensure_schema_matches<T: 'static>(schema: &SchemaHandle) -> Result<()> {
    let expected = TypeId::of::<T>();
    if schema.type_id() != expected {
        return Err(AgentError::Validation(format!(
            "schema `{}` does not match target type `{}`",
            schema.schema_name(),
            type_name::<T>(),
        )));
    }
    Ok(())
}
