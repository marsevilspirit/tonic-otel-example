use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

pub fn init_tracer_provider(resource: Resource) -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}
