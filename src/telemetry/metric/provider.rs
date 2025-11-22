use opentelemetry::global;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::MeterProviderBuilder;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;

pub fn init_meter_provider(resource: Resource) -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint("http://localhost:9090/api/v1/otlp/v1/metrics")
        .with_temporality(opentelemetry_sdk::metrics::Temporality::default())
        .build()
        .unwrap();

    let reader = PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(15))
        .build();

    let stdout_reader =
        PeriodicReader::builder(opentelemetry_stdout::MetricExporter::default()).build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource)
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}
