use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::KeyValue;

use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

use tonic_helloworld::telemetry::manager::init_telemetry_manager;
use tonic_helloworld::telemetry::metric::middleware::MetricLayer;
use tonic_helloworld::telemetry::trace::middleware::TraceLayer;

pub mod hello_world {
    tonic::include_proto!("helloworld"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct MyGreeter {
    request_counter: Counter<u64>,
}

impl MyGreeter {
    pub fn new() -> Self {
        let meter = global::meter("helloworld-server");
        let counter = meter.u64_counter("greeter.requests_total").init();
        Self {
            request_counter: counter,
        }
    }
}

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

        self.request_counter
            .add(1, &[KeyValue::new("key", "value")]);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let telemetry_manager = init_telemetry_manager("helloworld-server".to_string());

    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyGreeter::new();

    let layer = tower::ServiceBuilder::new()
        .layer(TraceLayer::new())
        .layer(MetricLayer::new())
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    telemetry_manager.shutdown();

    Ok(())
}
