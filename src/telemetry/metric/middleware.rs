use std::task::{Context, Poll};
use std::time::Instant;

use opentelemetry::global;
use opentelemetry::metrics::Histogram;

use tower::{Layer, Service};

#[derive(Clone)]
pub struct MetricLayer {
    histogram: Histogram<f64>,
}

impl MetricLayer {
    pub fn new() -> Self {
        let meter = global::meter("tonic_helloworld");
        let histogram = meter
            .f64_histogram("rpc.server.duration")
            .with_description("Duration of RPC server requests")
            .with_unit("ms")
            .init();
        Self { histogram }
    }
}

impl<S> Layer<S> for MetricLayer {
    type Service = MetricService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MetricService {
            inner: service,
            histogram: self.histogram.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MetricService<S> {
    inner: S,
    histogram: Histogram<f64>,
}

impl<S, Request, ResponseBody> Service<Request> for MetricService<S>
where
    S: Service<Request, Response = http::Response<ResponseBody>> + Send + 'static,
    S::Future: Send + 'static,
    ResponseBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let start = Instant::now();
        let histogram = self.histogram.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            let response = future.await;
            let duration = start.elapsed().as_secs_f64() * 1000.0;
            histogram.record(duration, &[]);
            response
        })
    }
}
