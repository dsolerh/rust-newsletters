use std::net::TcpListener;

use reqwest::Client;
use rstest::*;
use rust_newsletters::config::get_config;
use sqlx::{Connection, PgConnection};

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address.");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();

    let server = rust_newsletters::run(listener).expect("Failed to bind address");

    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    // We return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}

#[fixture]
fn setup_server_test() -> (String, Client) {
    (spawn_app(), Client::new())
}

#[rstest]
#[tokio::test]
async fn health_check_works(setup_server_test: (String, Client)) {
    // Arrange
    let (address, client) = setup_server_test;

    // Act
    let response = client
        .get(format!("{}/health-check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[rstest]
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data(setup_server_test: (String, Client)) {
    // Arrange
    let (address, client) = setup_server_test;
    let config = get_config().expect("Failed to read configuration.");
    let connection_string = config.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
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
    setup_server_test: (String, Client),
    #[case] body: String,
    #[case] error_message: String,
) {
    // Arrange
    let (address, client) = setup_server_test;

    // Act
    let response = client
        .post(format!("{}/subscriptions", address))
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
