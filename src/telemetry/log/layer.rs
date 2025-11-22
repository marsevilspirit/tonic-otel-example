use tracing::Subscriber;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::registry::LookupSpan;

pub fn init_log_layer<S>() -> Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
}
