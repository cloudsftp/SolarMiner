use anyhow::Error;
use dotenv::dotenv;
use nats_common::connect_nats;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;
    env_logger::init();

    let nats = connect_nats().await;

    Ok(())
}
