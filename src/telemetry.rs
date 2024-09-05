use tracing::{subscriber, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_tracing_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let tracing_env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let tracing_formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(tracing_env_filter)
        .with(JsonStorageLayer)
        .with(tracing_formatting_layer)
}

pub fn init_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all logs to tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
