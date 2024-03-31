use std::net::TcpListener;

use rust_newsletters::{config::get_config, startup::run};
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we can't read configuration
    let config = get_config().expect("Failed to read configuration.");

    // initialize the tracing
    LogTracer::init().expect("Failed to set logger");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new(
        "newsletters".into(),
        // Output the formatted spans to stdout. std::io::stdout
        std::io::stdout,
    );
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber.");

    // initialize the connection pool
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // initialize the listerner for the server
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    // run the server with the specified listener
    run(listener, connection_pool)?.await
}
