use std::str::FromStr;

use opentelemetry::{propagation::Injector, trace::TracerProvider};
use opentelemetry::KeyValue;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider, Resource};
use opentelemetry_semantic_conventions::{
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};

use tonic::metadata::{MetadataKey, MetadataMap};

use tracing::{info, instrument, Level};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::prelude::*;

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[instrument(fields(otel.kind = "client", otel.name = "test.helloworld/CallSayHello"))]
async fn call_service() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://127.0.0.1:50051").await?;

    let mut request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    opentelemetry::global::get_text_map_propagator(|propagator| {
        let context = tracing::Span::current().context();
        propagator.inject_context(&context, &mut MetadataInjector(request.metadata_mut()));
    });

    info!("inject_span_context, req: {:?}", request.metadata());
    // 检查 traceparent 是否存在
    if let Some(traceparent) = request.metadata().get("traceparent") {
        info!("traceparent header: {:?}", traceparent);
    }

    let response = client.say_hello(request).await?;

    info!("RESPONSE={:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();
    call_service().await
}

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    Resource::builder()
        .with_service_name("helloworld-client")
        .with_schema_url(
            [
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
            ],
            SCHEMA_URL,
        )
        .build()
}

fn init_tracer_provider() -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource())
        .build()
}

fn init_tracing_subscriber() -> OtelGuard {
    let tracer_provider = init_tracer_provider();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = tracer_provider.tracer("tracing-otel-subscriber");

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from_level(
            Level::INFO,
        ))
        .with(OpenTelemetryLayer::new(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();

    OtelGuard {
        tracer_provider: tracer_provider,
    }
}

struct OtelGuard {
    tracer_provider: SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("{err:?}");
        }
    }
}

// 实现 TextMap 接口以注入 metadata
pub struct MetadataInjector<'a>(&'a mut MetadataMap);

impl<'a> Injector for MetadataInjector<'a> {
    /// Set a key and value in the MetadataMap.  Does nothing if the key or value are not valid inputs
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = MetadataKey::from_str(key) {
            if let Ok(val) = value.parse() {
                self.0.append(key, val);
            }
        }
    }
}