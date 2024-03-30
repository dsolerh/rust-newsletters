use std::net::TcpListener;

use reqwest::Client;
use rstest::*;
use rust_newsletters::{
    config::{get_config, DatabaseSettings},
    startup,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};

struct TestApp {
    address: String,
    db_pool: PgPool,
}

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
async fn spawn_test_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address.");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();

    let mut config = get_config().expect("Failed to read configuration.");
    
    let connection_pool = configure_database(&mut config.database).await;

    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");

    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    // We return the test data
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &mut DatabaseSettings) -> PgPool {
    config.generate_database_name();
    // create the database
    let mut connection = PgConnection::connect(&config.connection_string_without_db_name())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create the database.");

    // Migrate the database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}

#[fixture]
async fn setup_server_test() -> (TestApp, Client) {
    (spawn_test_app().await, Client::new())
}

#[rstest]
#[tokio::test]
async fn health_check_works(#[future] setup_server_test: (TestApp, Client)) {
    // Arrange
    let (app, client) = setup_server_test.await;

    // Act
    let response = client
        .get(format!("{}/health-check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[rstest]
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data(#[future] setup_server_test: (TestApp, Client)) {
    // Arrange
    let (app, client) = setup_server_test.await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[rstest]
#[case::missing_email("name=le%20guin", "missing the email")]
#[case::missing_name("email=ursula_le_guin%40gmail.com", "missing the name")]
#[case::empty_body("", "missing both name and email")]
// #[case::empty_email("name=daniel&email= ", "empty email")]
#[tokio::test]
async fn subscribe_returns_400_when_form_data_is_missing(
    #[future] setup_server_test: (TestApp, Client),
    #[case] body: String,
    #[case] error_message: String,
) {
    // Arrange
    let (app, client) = setup_server_test.await;

    // Act
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        // Additional customised error message on test failure
        "The API did not fail with 400 Bad Request when the payload was {}.",
        error_message
    );
}
