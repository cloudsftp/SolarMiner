use std::env;

use anyhow::{Context, Error, anyhow};
use dotenv::dotenv;
use futures_util::StreamExt;
use log::debug;
use nats_common::{MessageStream, connect_jetstream, create_stream, try_pub_sub_subscribe};
use tokio::signal;

mod sitedata;

const STATE_STREAM: &str = "controller-state";
const CONTROLLER_COMMANDS_STREAM: &str = "controller-commands";

#[derive(Debug, Clone)]
struct Config {
    prod: bool,
    state_stream_name: String,
    controller_commands_stream_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    let config = if env::var("PROD").is_ok_and(|value| value == "true") {
        Config {
            prod: true,
            state_stream_name: STATE_STREAM.into(),
            controller_commands_stream_name: CONTROLLER_COMMANDS_STREAM.into(),
        }
    } else {
        Config {
            prod: false,
            state_stream_name: format!("dev-{}", STATE_STREAM),
            controller_commands_stream_name: format!("dev-{}", CONTROLLER_COMMANDS_STREAM),
        }
    };

    let main_task = tokio::spawn(run(config.clone()));

    tokio::select! {
        Ok(_) = signal::ctrl_c() => {},
        Err(_) = signal::ctrl_c() => {
            eprintln!("Could not listen to sigterm")
        },
        result = main_task => {match result {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Main task errored")
            },
        }},
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

async fn run(config: Config) -> Result<(), Error> {
    let js = connect_jetstream().await;
    let state_stream = create_stream(&js, &config.state_stream_name).await;
    let mut state_messages: MessageStream = try_pub_sub_subscribe(&js, &config.state_stream_name)
        .await
        .map_err(|err| anyhow!(err)) // TODO: remove as soon as library has better errors
        .context("could not subscribe to controller state stream")?;

    let controller_command_stream =
        create_stream(&js, &config.controller_commands_stream_name).await;

    match js
        .publish(config.controller_commands_stream_name, "hello".into())
        .await
    {
        Ok(_) => println!("Ok"),
        Err(err) => panic!("{}", err),
    }

    while let Some(message) = state_messages.next().await {
        debug!("Received message {:?}", message);
    }

    Ok(())
}
