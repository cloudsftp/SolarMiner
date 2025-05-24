use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions, Message,
    jetstream::{self, Context, stream},
};
use dotenv::dotenv;
use futures::{Stream, StreamExt, future::try_join_all, stream::select_all};
use log::{debug, error, info};
use serde::Deserialize;
use serde_json::from_reader;
use std::{env, fs::File, io::BufReader};
use tokio::signal::unix::{self, SignalKind};

mod plug;
mod state;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    state_stream_name: String,
    controller_commands_stream_name: String,
    plug_name: String,
}

use state::State;

async fn run(config: Config, pi_nats: Client, server_js: Context) -> Result<(), Error> {
    server_js
        .create_or_update_stream(stream::Config {
            name: config.state_stream_name.clone(),
            ..Default::default()
        })
        .await
        .context("Could not create the state stream for the service")?;

    let mut pi_messages = nats_subscribe(
        &pi_nats,
        &[
            "stat.*.RESULT",
            "solaredge.modbus.battery.battery0",
            "solaredge.powerflow",
        ],
    )
    .await
    .context("Could not subscribe to the subjects on the controller")?;

    pi_nats
        .publish("cmnd.plug_bitaxe_001.Power", "".into())
        .await?;

    let mut state = State::new(config);
    while let Some(message) = pi_messages.next().await {
        state = state.handle_message(message, &pi_nats, &server_js).await?;
    }

    Ok(())
}

async fn nats_subscribe(
    nats: &Client,
    subjects: &[&str],
) -> Result<impl Stream<Item = Message>, Error> {
    let subscribers = try_join_all(
        subjects
            .iter()
            .map(async |subject| nats.subscribe(subject.to_string()).await),
    )
    .await?;

    Ok(select_all(subscribers))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    /*
    let state_stream_name: &str = get_env("STATE_STREAM_NAME")?.leak();
    let controller_commands_stream_name: &str = get_env("CONTROLLER_COMMANDS_STREAM_NAME")?.leak();

    let config = Config {
        state_stream_name,
        controller_commands_stream_name,
    };
    */
    let config_file = File::open("config.json")?;
    let config = from_reader(BufReader::new(config_file))?;

    let pi_nats = connect_nats_client("PI").await?;

    let server_nats = connect_nats_client("SERVER").await?;
    let server_js = jetstream::new(server_nats);

    let main_task = tokio::spawn(run(config, pi_nats, server_js)); // TODO: wrap communications in struct, extra thread for sending?

    // TODO: second task probing energy periodically (e.g. 1h)

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

    Ok(())
}

async fn connect_nats_client(prefix: &str) -> Result<Client, Error> {
    let get_env = |name| get_env(&format!("{}_{}", prefix, name));

    let host = get_env("NATS_HOST")?;
    let port = get_env("NATS_PORT")?;
    let options = ConnectOptions::new().token(get_env("NATS_TOKEN")?);

    options
        .connect(format!("{}:{}", host, port))
        .await
        .context(format!("Could not connect to nats server '{}'", prefix))
}

fn get_env(key: &str) -> Result<String, Error> {
    env::var(key).context(format!("could not get value for key '{}'", key))
}
