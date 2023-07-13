use tracing::Subscriber;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// # Notes
//
// We're using `impl Subscriber` as return type to avoid having to
// spell out the actual type of the returned subscriber, which is
// indeed quite compiles.

pub fn get_debug_subscriber(env_filter: &str) -> impl Subscriber + Sync + Send {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty();

    Registry::default().with(env_filter).with(formatting_layer)
}

pub fn get_plain_subscriber(env_filter: &str) -> impl Subscriber + Sync + Send {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(false)
        .with_level(true)
        .compact();

    Registry::default().with(env_filter).with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once.
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("failed to set subscriber");
}
