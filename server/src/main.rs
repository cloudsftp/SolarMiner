use std::env;

use anyhow::{Context, Error, anyhow};
use dotenv::dotenv;
use env_logger::Target;
use futures_util::StreamExt;
use log::{debug, error, info};
use nats_common::{MessageStream, connect_jetstream, create_stream, try_pub_sub_subscribe};
use tokio::signal::unix::{self, SignalKind};

mod sitedata;

#[derive(Debug, Clone, Copy)]
struct Config {
    state_stream_name: &'static str,
    controller_commands_stream_name: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::new().target(Target::Stdout).init();
    dotenv()?;

    let state_stream_name: &str = env::var("STATE_STREAM_NAME")?.leak();
    let controller_commands_stream_name: &str = env::var("CONTROLLER_COMMANDS_STREAM_NAME")?.leak();

    let config = Config {
        state_stream_name,
        controller_commands_stream_name,
    };

    let main_task = tokio::spawn(run(config));

    let mut signal_terminate = unix::signal(SignalKind::terminate())?;
    tokio::select! {
        _ = signal_terminate.recv() => {},
        result = main_task => {
            match result {
                Ok(Ok(())) => {
                    info!("Main task exited successfully")
                },
                Ok(Err(err)) => {
                    error!("Main task errored: {}", err)
                },
                Err(err) => {
                    error!("Could not join main task: {}", err)
                },
            }
        }
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
