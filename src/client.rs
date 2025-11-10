use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::{
    SCHEMA_URL,
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_VERSION},
};

use tracing::{Level, info, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::prelude::*;

use hello_world::HelloRequest;
use hello_world::greeter_client::GreeterClient;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    Resource::builder()
        .with_service_name(env!("CARGO_PKG_NAME"))
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

#[instrument]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();

    let mut client = GreeterClient::connect("http://127.0.0.1:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    info!("RESPONSE={:?}", response);

    Ok(())
}
