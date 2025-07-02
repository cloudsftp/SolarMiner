use anyhow::Error;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;
    env_logger::init();

    Ok(())
}
