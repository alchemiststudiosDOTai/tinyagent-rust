#![allow(dead_code)]

use crate::schemas::{CompletionSchema, SchemaHandle};
use serde_json::{json, Value};

#[derive(Clone, Debug, Default)]
pub struct SchemaContext {
    active: Option<SchemaHandle>,
}

impl SchemaContext {
    pub fn set_handle(&mut self, handle: SchemaHandle) {
        self.active = Some(handle);
    }

    pub fn set<T: CompletionSchema>(&mut self) {
        self.active = Some(T::schema().clone());
    }

    pub fn clear(&mut self) {
        self.active = None;
    }

    pub fn handle(&self) -> Option<&SchemaHandle> {
        self.active.as_ref()
    }

    pub fn response_format(&self) -> Option<Value> {
        self.active.as_ref().map(|handle| {
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": handle.schema_name(),
                    "schema": handle.schema_json()
                }
            })
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Conversation {
    messages: Vec<Value>,
    schema: SchemaContext,
}

impl Conversation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_messages(messages: Vec<Value>) -> Self {
        Self {
            messages,
            schema: SchemaContext::default(),
        }
    }

    pub fn push_message(&mut self, message: Value) {
        self.messages.push(message);
    }

    pub fn extend_messages(&mut self, additional: impl IntoIterator<Item = Value>) {
        self.messages.extend(additional);
    }

    pub fn messages(&self) -> &[Value] {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut Vec<Value> {
        &mut self.messages
    }

    pub fn schema_context(&self) -> &SchemaContext {
        &self.schema
    }

    pub fn schema_context_mut(&mut self) -> &mut SchemaContext {
        &mut self.schema
    }

    pub fn set_schema_handle(&mut self, handle: SchemaHandle) {
        self.schema.set_handle(handle);
    }

    pub fn set_schema<T: CompletionSchema>(&mut self) {
        self.schema.set::<T>();
    }

    pub fn clear_schema(&mut self) {
        self.schema.clear();
    }

    pub fn response_format(&self) -> Option<Value> {
        self.schema.response_format()
    }
}
