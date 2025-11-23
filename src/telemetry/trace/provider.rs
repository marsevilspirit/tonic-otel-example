use opentelemetry_sdk::trace::TracerProvider as SdkTracerProvider;
use opentelemetry_sdk::Resource;

pub fn init_tracer_provider(resource: Resource) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .build_span_exporter()
        .unwrap();

    let config = opentelemetry_sdk::trace::Config::default().with_resource(resource);
    opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_config(config)
        .build()
}
