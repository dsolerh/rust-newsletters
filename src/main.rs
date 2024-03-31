use rust_newsletters::{
    config::get_config,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we can't read configuration
    let config = get_config().expect("Failed to read configuration.");

    // initialize the tracing
    let subscriber = get_subscriber("newsletters".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

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
