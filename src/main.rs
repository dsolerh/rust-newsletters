use rust_newsletters::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    run().await
}
