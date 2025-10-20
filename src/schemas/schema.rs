use schemars::schema::{ObjectValidation, RootSchema, Schema, SchemaObject};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    any::{type_name, TypeId},
    sync::Arc,
};

/// Cached JSON schema handle associated with a response type.
#[derive(Clone, Debug)]
pub struct SchemaHandle {
    schema_name: &'static str,
    type_name: &'static str,
    type_id: TypeId,
    schema_json: Arc<Value>,
}

impl SchemaHandle {
    pub fn from_root_schema<T: 'static>(
        schema_name: &'static str,
        type_name: &'static str,
        root: RootSchema,
    ) -> Self {
        let schema_json = serde_json::to_value(root)
            .unwrap_or_else(|err| panic!("failed to serialize schema for {}: {}", type_name, err));

        Self {
            schema_name,
            type_name,
            type_id: TypeId::of::<T>(),
            schema_json: Arc::new(schema_json),
        }
    }

    pub fn schema_name(&self) -> &'static str {
        self.schema_name
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn schema_json(&self) -> &Value {
        self.schema_json.as_ref()
    }

    pub fn schema_json_arc(&self) -> Arc<Value> {
        Arc::clone(&self.schema_json)
    }
}

pub trait CompletionSchema: DeserializeOwned + Send + Sync + 'static {
    fn schema() -> &'static SchemaHandle;
}

/// Apply doc comments captured by the procedural macro to the generated schema metadata.
pub fn apply_doc_comments(
    root: &mut RootSchema,
    title: &'static str,
    description: Option<&'static str>,
    field_docs: &[(&'static str, &'static str)],
) {
    let schema_object = &mut root.schema;
    apply_struct_metadata(schema_object, title, description);

    if let Some(object_validation) = schema_object.object.as_mut() {
        apply_field_metadata(object_validation.as_mut(), field_docs);
    }
}

fn apply_struct_metadata(
    schema_object: &mut SchemaObject,
    title: &'static str,
    description: Option<&'static str>,
) {
    let metadata = schema_object.metadata();

    if metadata.title.is_none() {
        metadata.title = Some(title.to_string());
    }

    if let Some(description) = description {
        if metadata.description.is_none() {
            metadata.description = Some(description.to_string());
        }
    }
}

fn apply_field_metadata(
    object_validation: &mut ObjectValidation,
    field_docs: &[(&'static str, &'static str)],
) {
    for (field, doc) in field_docs {
        if let Some(Schema::Object(field_object)) = object_validation.properties.get_mut(*field) {
            let metadata = field_object.metadata();
            if metadata.description.is_none() {
                metadata.description = Some((*doc).to_string());
            }
        }
    }
}

/// Helper so callers can retrieve the Rust type name of a schema provider.
pub fn schema_type_name<T>() -> &'static str {
    type_name::<T>()
}
