use std::str::FromStr;

use opentelemetry::global;
use opentelemetry::propagation::Injector;
use tonic::metadata::MetadataKey;
use tonic::metadata::MetadataMap;
use tonic::transport::Endpoint;
use tonic::Request;
use tonic::Status;
use tracing::info;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use tonic_helloworld::telemetry::manager::init_telemetry_manager;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[instrument(fields(
    otel.kind = "client",
    otel.name = "test.helloworld/CallSayHello",
    rpc.system = "grpc",
))]
async fn call_service() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::from_str("http://127.0.0.1:50051").unwrap();
    let channel = endpoint.connect().await?;

    let mut client = GreeterClient::with_interceptor(channel, send_trace);

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    info!("RESPONSE={:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let telemetry_manager = init_telemetry_manager("helloworld-client".to_string());
    let _ = call_service().await;
    telemetry_manager.shutdown();
    Ok(())
}

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

pub fn send_trace<T>(mut request: Request<T>) -> Result<Request<T>, Status> {
    global::get_text_map_propagator(|propagator| {
        let context = tracing::Span::current().context();
        propagator.inject_context(&context, &mut MetadataInjector(request.metadata_mut()))
    });

    Ok(request)
}
