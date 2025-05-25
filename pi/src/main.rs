use anyhow::{Context as AnyhowContext, Error};
use async_nats::jetstream::stream;
use dotenv::dotenv;
use futures::StreamExt;
use log::{error, info};
use serde::Deserialize;
use serde_json::from_reader;
use std::{fs::File, io::BufReader};
use tokio::signal::unix::{self, SignalKind};

mod communication;
mod state;

use communication::{Communication, nats_subscribe};
use state::State;

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

impl Config {
    fn from_file(file_name: &str) -> Result<Self, Error> {
        let config_file =
            File::open(file_name).context(format!("Could not open config file '{}'", file_name))?;
        let config_file = BufReader::new(config_file);
        from_reader(config_file).context(format!("Could not parse config file '{}'", file_name))
    }
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
            self.comm.pi_nats.clone(),
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

        // TODO: also listen to
        // - commands
        // - timer for aggregating power data and sending it out
        // TODO: main loop should never stop, send out error message and continue
        while let Some(message) = pi_messages.next().await {
            self.update_state(&message).await?;

            // Perform Action TODO: move to extra function
            let on = self.mining_condition();
            self.flip_plug_switch(on).await?;
        }

        Ok(())
    }

    async fn flip_plug_switch(&self, on: bool) -> Result<(), Error> {
        if self.plug_state_satisfied(on) {
            return Ok(());
        }

        let payload = if on { "ON" } else { "OFF" }.into();

        self.comm
            .pi_nats
            .publish(format!("cmnd.{}.POWER", self.config.plug_name), payload)
            .await?;

        Ok(())
    }
}

impl App {
    async fn init() -> Result<Self, Error> {
        let config = Config::from_file("config.json")?;

        let state = State::default();

        let comm = Communication::connect()
            .await
            .context("Could not connect to the communication services")?;

        Ok(App {
            config,
            state,
            comm,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

    let app = App::init().await?;
    let main_task = tokio::spawn(app.run());

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
