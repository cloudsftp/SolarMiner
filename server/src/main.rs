use anyhow::{Context, Error, anyhow};
use dotenv::dotenv;
use futures_util::StreamExt;
use nats_common::{MessageStream, connect_jetstream, create_stream, try_pub_sub_subscribe};

mod sitedata;

const STATE_STREAM: &str = "controller-state";
const CONTROLLER_COMMANDS_STREAM: &str = "controller-commands";

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;

    let js = connect_jetstream().await;

    let state_stream = create_stream(&js, STATE_STREAM).await;
    let mut state_messages: MessageStream = try_pub_sub_subscribe(&js, STATE_STREAM)
        .await
        .map_err(|err| anyhow!(err)) // TODO: remove as soon as library has better errors
        .context("could not subscribe to controller state stream")?;

    let controller_command_stream = create_stream(&js, CONTROLLER_COMMANDS_STREAM).await;

    while let Some(message) = state_messages.next().await {
        todo!()
    }

    /*
        let api_key = env::var("API_KEY")?;
        let site_id = env::var("SITE_ID")?;

        let data_provider = SolarEdgeDataProvider::new(api_key, site_id);

        let excess_power = data_provider.get_current_excess_power().await?;

        dbg!(excess_power);

        let base_url = format!("{}{}/", API_URL, site_id);
        let url = format!("{}currentPowerFlow?api_key={}", base_url, api_key);
    */

    Ok(())
}
