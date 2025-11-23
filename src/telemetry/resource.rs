use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::DEPLOYMENT_ENVIRONMENT_NAME;
use opentelemetry_semantic_conventions::resource::SERVICE_VERSION;

// Create a Resource that captures information about the entity for which telemetry is recorded.
pub fn resource(service_name: String) -> Resource {
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name,
        ),
        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
    ])
}
