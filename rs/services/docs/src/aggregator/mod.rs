use crate::config::{ApiConfig, ServiceConfig};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum AggregatorError {
    #[error("http request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("failed to parse openapi spec: {0}")]
    ParseFailed(#[from] serde_json::Error),
}

pub struct Aggregator {
    client: reqwest::Client,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Update all $ref references and tags in a JSON value to include service prefix
    fn update_refs_and_tags(value: &mut Value, service_name: &str) {
        match value {
            Value::Object(map) => {
                // Check if this object has a $ref key
                if let Some(ref_value) = map.get("$ref")
                    && let Some(ref_str) = ref_value.as_str()
                {
                    // Update schema references to include service prefix
                    if ref_str.starts_with("#/components/schemas/") {
                        let schema_name = ref_str.replace("#/components/schemas/", "");
                        let prefixed_ref =
                            format!("#/components/schemas/{}_{}", service_name, schema_name);
                        map.insert("$ref".to_string(), Value::String(prefixed_ref));
                    }
                }
                // Check if this object has a tags array and update tag names
                if let Some(tags_value) = map.get_mut("tags")
                    && let Some(tags_array) = tags_value.as_array_mut()
                {
                    for tag in tags_array.iter_mut() {
                        if let Some(tag_str) = tag.as_str() {
                            // Skip health tag
                            if tag_str == "health" {
                                continue;
                            }
                            // Prefix tag with service name
                            let prefixed_tag = format!("{}: {}", service_name, tag_str);
                            *tag = Value::String(prefixed_tag);
                        }
                    }
                }
                // Recursively update all nested objects
                for (_, v) in map.iter_mut() {
                    Self::update_refs_and_tags(v, service_name);
                }
            }
            Value::Array(arr) => {
                // Recursively update all array elements
                for item in arr.iter_mut() {
                    Self::update_refs_and_tags(item, service_name);
                }
            }
            _ => {}
        }
    }

    /// Fetch OpenAPI spec from a single service
    pub async fn fetch_spec(&self, service: &ServiceConfig) -> Result<Value, AggregatorError> {
        let openapi_url = service.openapi_url();
        tracing::info!(
            "Fetching OpenAPI spec from {}: {}",
            service.name,
            openapi_url
        );

        let response = self.client.get(&openapi_url).send().await?;
        let spec = response.json::<Value>().await?;

        Ok(spec)
    }

    /// Fetch all OpenAPI specs from configured services
    /// Returns a map of service name to spec (skips failed fetches)
    pub async fn fetch_all_specs(&self, services: &[ServiceConfig]) -> HashMap<String, Value> {
        let mut specs = HashMap::new();

        for service in services {
            match self.fetch_spec(service).await {
                Ok(spec) => {
                    tracing::info!("Successfully fetched spec for {}", service.name);
                    specs.insert(service.name.clone(), spec);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch spec for {}: {}", service.name, e);
                }
            }
        }

        specs
    }

    /// Merge multiple OpenAPI specs into a single spec
    /// Tags are prefixed with service name to avoid conflicts
    /// Includes the API gateway URL in the servers field
    pub fn merge_specs(&self, specs: HashMap<String, Value>, api_config: &ApiConfig) -> Value {
        let mut merged = serde_json::json!({
            "openapi": "3.1.0",
            "info": {
                "title": api_config.title,
                "version": api_config.version,
                "description": api_config.description
            },
            "servers": [
                {
                    "url": api_config.url
                }
            ],
            "paths": {},
            "components": {
                "schemas": {}
            },
            "tags": []
        });

        for (service_name, mut spec) in specs {
            // Merge paths with service name prefix
            if let Some(paths) = spec.get_mut("paths")
                && let Some(merged_paths) = merged["paths"].as_object_mut()
                && let Some(spec_paths) = paths.as_object()
            {
                for (path, operations) in spec_paths {
                    // Skip health endpoints - API gateway will provide its own
                    if path == "/health" {
                        continue;
                    }
                    // Update schema references and tags in operations
                    let mut operations = operations.clone();
                    Self::update_refs_and_tags(&mut operations, &service_name);

                    // Prefix path with service name (e.g., /people becomes /auth/people)
                    let prefixed_path = format!("/{}{}", service_name, path);
                    merged_paths.insert(prefixed_path, operations);
                }
            }

            // Merge component schemas
            if let Some(components) = spec.get_mut("components")
                && let Some(schemas) = components.get_mut("schemas")
                && let Some(merged_schemas) = merged["components"]["schemas"].as_object_mut()
                && let Some(spec_schemas) = schemas.as_object()
            {
                for (schema_name, schema) in spec_schemas {
                    // Prefix schema names with service name to avoid conflicts
                    let prefixed_name = format!("{}_{}", service_name, schema_name);
                    merged_schemas.insert(prefixed_name, schema.clone());
                }
            }

            // Merge tags with service name prefix
            if let Some(tags) = spec.get_mut("tags")
                && let Some(merged_tags) = merged["tags"].as_array_mut()
                && let Some(spec_tags) = tags.as_array()
            {
                for tag in spec_tags {
                    // Skip health tag
                    if let Some(tag_obj) = tag.as_object()
                        && let Some(name) = tag_obj.get("name")
                        && name.as_str() == Some("health")
                    {
                        continue;
                    }

                    let mut tag = tag.clone();
                    if let Some(tag_obj) = tag.as_object_mut()
                        && let Some(name) = tag_obj.get("name")
                    {
                        let prefixed_name =
                            format!("{}: {}", service_name, name.as_str().unwrap_or(""));
                        tag_obj.insert("name".to_string(), Value::String(prefixed_name));
                    }
                    merged_tags.push(tag);
                }
            }
        }

        merged
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}
