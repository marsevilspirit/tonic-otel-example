use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::{
    attribute::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};

use tracing::{info, instrument, Level, info_span};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::prelude::*;

use tonic::{metadata::MetadataMap, transport::Server, Request, Response, Status};
use http;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    #[instrument(
        name = "test.helloworld/SayHello",
        fields(otel.kind = "server"),
        skip(self, request))]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        info!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();

    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .trace_fn(|request: &http::Request<()>| {
            let headers = request.headers();
            
            // 提取追踪上下文
            let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.extract(&opentelemetry_http::HeaderExtractor(headers))
            });
            
            // 将父上下文附加到当前线程
            // 这样后续创建的 span 会自动关联到父上下文
            let _guard = parent_context.attach();
            
            // 创建 span，它会自动使用已附加的父上下文
            let span = info_span!("grpc_request");
            
            tracing::debug!("Extracted trace context from request headers");
            
            span
        })
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    Resource::builder()
        .with_service_name("helloworld-server")
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

struct MetadataInjector<'a>(&'a mut MetadataMap);

impl<'a> opentelemetry::propagation::Injector for MetadataInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

// 实现 Extractor trait 以从 metadata 中提取追踪上下文
struct MetadataExtractor<'a>(&'a MetadataMap);

impl<'a> opentelemetry::propagation::Extractor for MetadataExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        // keys() 方法对于 OpenTelemetry propagator 通常不是必需的
        // 返回空向量即可，因为 propagator 主要使用 get() 方法
        Vec::new()
    }
}
