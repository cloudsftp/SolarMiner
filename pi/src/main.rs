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

mod communication;
mod state;

use communication::Communication;
use state::{PlugState, State};

// TODO: App in extra module?
#[derive(Debug)]
struct App {
    pub config: Config,
    pub state: State,
    pub comm: Communication,
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    state_stream_name: String,
    controller_commands_stream_name: String,
    plug_name: String,
    miner_demand: usize,
}

impl App {
    async fn run(mut self) -> Result<(), Error> {
        self.comm
            .server_js
            .create_or_update_stream(stream::Config {
                name: self.config.state_stream_name.clone(),
                ..Default::default()
            })
            .await
            .context("Could not create the state stream for the service")?;

        let mut pi_messages = nats_subscribe(
            &self.comm.pi_nats,
            &[
                "stat.*.RESULT",
                "solaredge.modbus.battery.battery0",
                "solaredge.powerflow",
            ],
        )
        .await
        .context("Could not subscribe to the subjects on the controller")?;

        self.comm
            .pi_nats
            .publish(format!("cmnd.{}.Power", self.config.plug_name), "".into())
            .await?;

        while let Some(message) = pi_messages.next().await {
            self.state = self.state.handle_message(&self.config, &message).await?;

            // Perform Action TODO: move to extra function

            let on = self // TODO: wrap condition in method on State
                .state
                .production_to_grid
                .is_some_and(|production| production > self.config.miner_demand);

            self.flip_plug_switch(on).await?;
        }

        Ok(())
    }

    async fn flip_plug_switch(&self, on: bool) -> Result<(), Error> {
        if (on && self.state.plug_state == PlugState::On)
            || (!on && self.state.plug_state == PlugState::Off)
        {
            return Ok(());
        }

        let payload = if on { "ON" } else { "OFF" }.into();

        self.comm
            .pi_nats
            .publish(format!("cmnd.{}.POWER", self.config.plug_name), payload)
            .await?;

        return Ok(());
    }
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

    let config_file = File::open("config.json")?;
    let config = from_reader(BufReader::new(config_file))?;

    let state = State::default();

    let comm = Communication::connect()
        .await
        .context("Could not connect to the communication services")?;

    let app = App {
        config,
        state,
        comm,
    };

    let main_task = tokio::spawn(app.run()); // TODO: wrap communications in struct, extra thread for sending?

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
