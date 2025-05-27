use std::time::Duration;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::jetstream::stream;
use config::Config;
use controller::Controller;
use dotenv::dotenv;
use futures::StreamExt;
use log::{error, info};
use once_cell::sync::Lazy;
use tokio::{
    signal::unix::{self, SignalKind},
    time::interval,
};

mod communication;
mod config;
mod controller;
mod state;

use communication::{Communication, nats_subscribe};
use state::State;

#[derive(Debug)]
struct App {
    state: State,
    controller: Controller,
    comm: Communication,
}

static CONFIG: Lazy<Config> =
    Lazy::new(|| Config::from_file("config.yaml").expect("Could not load config"));

impl App {
    async fn run(mut self) -> Result<(), Error> {
        self.comm
            .server_js
            .create_or_update_stream(stream::Config {
                name: CONFIG.communication.state_stream_name.clone(),
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
            .publish(
                format!("cmnd.{}.Power", CONFIG.communication.plug_name),
                "".into(),
            )
            .await?;

        // TODO: also listen to
        // - commands (nats)
        // - timer for controlling
        // - timer for aggregating power data and sending it out
        //

        let mut controlling_interval =
            interval(Duration::from_secs_f32(CONFIG.controller.controller_time));

        loop {
            tokio::select! {
                Some(message) = pi_messages.next() => {
                    if let Err(err) = self.state.update(&message).await {
                        // TODO: send out error message and continue
                        error!("Errored while updating the state: {}", err);
                        continue;
                    }
                }
                _ = controlling_interval.tick() => {
                    if let Err(err) = self.perform_control_action().await {
                        error!("Errored while flipping the miner plug: {}", err);
                        continue;
                    }
                }
            }
        }
    }
}

impl App {
    async fn init() -> Result<Self, Error> {
        let state = State::new(&CONFIG);
        let comm = Communication::connect()
            .await
            .context("Could not connect to the communication services")?;

        Ok(App { state, comm })
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
