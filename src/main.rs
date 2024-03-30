use std::net::TcpListener;

use rust_newsletters::{config::get_config, startup::run};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we can't read configuration
    let config = get_config().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", config.application_port);
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let listener = TcpListener::bind(address)?;
    // run the server with the specified listener
    run(listener, connection_pool)?.await
}
