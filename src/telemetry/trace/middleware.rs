use std::task::{Context, Poll};

use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;
use tower::{Layer, Service};
use tracing::{info_span, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Clone)]
pub struct TraceLayer;

impl TraceLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        TraceService { inner: service }
    }
}

#[derive(Clone)]
pub struct TraceService<S> {
    inner: S,
}

impl<S, RequestBody, ResponseBody> Service<http::Request<RequestBody>> for TraceService<S>
where
    S: Service<http::Request<RequestBody>, Response = http::Response<ResponseBody>>
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<RequestBody>) -> Self::Future {
        let parent_context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(req.headers()))
        });
        let span = info_span!("helloworld-service");
        let _ = span.set_parent(parent_context);

        let future = self.inner.call(req);

        Box::pin(async move { future.await }.instrument(span))
    }
}
