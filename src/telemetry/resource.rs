use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::DEPLOYMENT_ENVIRONMENT_NAME;
use opentelemetry_semantic_conventions::resource::SERVICE_VERSION;
use opentelemetry_semantic_conventions::SCHEMA_URL;

// Create a Resource that captures information about the entity for which telemetry is recorded.
pub fn resource(service_name: String) -> Resource {
    Resource::builder()
        .with_service_name(service_name)
        .with_schema_url(
            [
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
            ],
            SCHEMA_URL,
        )
        .build()
}
