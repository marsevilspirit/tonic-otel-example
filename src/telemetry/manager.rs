use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::Level;
use tracing_opentelemetry::MetricsLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::prelude::*;

use crate::telemetry::log::layer::init_log_layer;
use crate::telemetry::metric::provider::init_meter_provider;
use crate::telemetry::resource::resource;
use crate::telemetry::trace::provider::init_tracer_provider;

pub struct TelemetryManager {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl TelemetryManager {
    fn new(
        tracer_provider: SdkTracerProvider,
        meter_provider: SdkMeterProvider,
    ) -> TelemetryManager {
        Self {
            tracer_provider,
            meter_provider,
        }
    }
    pub fn shutdown(&self) {
        let _ = self.tracer_provider.shutdown();
        let _ = self.meter_provider.shutdown();
    }
}

pub fn init_telemetry_manager(service_name: String) -> TelemetryManager {
    let resource = resource(service_name);

    let filter = tracing_subscriber::filter::LevelFilter::from_level(Level::INFO);

    let tracer_provider = init_tracer_provider(resource.clone());
    let tracer = tracer_provider.tracer("tonic-otel-example");
    let tracer_layer = OpenTelemetryLayer::new(tracer);

    let meter_provider = init_meter_provider(resource);
    let meter_layer = MetricsLayer::new(meter_provider.clone());

    let log_layer = init_log_layer();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(filter)
        .with(tracer_layer)
        .with(meter_layer)
        .with(log_layer)
        .init();

    TelemetryManager::new(tracer_provider, meter_provider)
}
