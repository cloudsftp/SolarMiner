use std::time::Duration;

use anyhow::{Context as AnyhowContext, Error};
use config::Config;
use controller::Controller;
use dotenv::dotenv;
use env_logger::Target;
use futures::StreamExt;
use log::{error, info};
use once_cell::sync::Lazy;
use tokio::{
    signal::unix::{self, SignalKind},
    time::{Instant, MissedTickBehavior, interval, interval_at},
};

mod communication;
mod config;
mod controller;
mod state;

use communication::Communication;
use state::PartialState;

#[derive(Debug)]
struct App {
    state: PartialState,
    controller: Controller,
}

static CONFIG: Lazy<Config> =
    Lazy::new(|| Config::from_file("config.yaml").expect("Could not load config"));

impl App {
    async fn run(mut self, comm: Communication) -> Result<(), Error> {
        comm.create_service_streams().await?;
        let mut update_events = comm.get_update_events().await?;

        let create_action_interval = |offset: u64| -> Result<_, Error> {
            let mut interval = interval_at(
                Instant::now()
                    .checked_add(CONFIG.controller.sensor_data_update_interval)
                    .context("Action interval start time not in range")?
                    .checked_add(Duration::from_secs(offset))
                    .context("Action interval start time not in range")?,
                CONFIG.controller.controller_interval,
            );
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            Ok(interval)
        };

        let mut perform_control_action = create_action_interval(0)?;
        let mut report_state = create_action_interval(1)?;

        let create_sensor_data_update_interval = || {
            let mut interval = interval(CONFIG.controller.sensor_data_update_interval);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            interval
        };

        let mut query_miner_state = create_sensor_data_update_interval();
        let mut query_inverter_state = create_sensor_data_update_interval();

        loop {
            tokio::select! {
                _ = query_inverter_state.tick() => {
                    // TODO: implement reading from modbus and then sending update on a update_events channel
                }
                _ = query_miner_state.tick() => {
                    if let Err(err) = comm.query_plug_state().await {
                        error!("Errored while querying the plug state: {}", err);
                        continue;
                    }
                }
                // TODO: instead create update_events stream and listen to that
                // that stream icludes pi_messages mapped to UpdateEvents
                // and inverter queries also mapped to update events
                Some(update_event) = update_events.next() => {
                    if let Err(err) = self.state.update(update_event).await {
                        // TODO: send out error message and continue
                        error!("Errored while updating the state: {}", err);
                        continue;
                    }
                }
                _ = perform_control_action.tick() => {
                    if let Err(err) = self.controller.perform_action(&self.state, &comm).await {
                        error!("Errored while flipping the miner plug: {}", err);
                        continue;
                    }
                }
                _ = report_state.tick() => {
                    // Implement reporting
                    ()
                }
            }
        }
    }
}

impl App {
    fn new() -> Self {
        App {
            state: PartialState::new(),
            controller: Controller::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;
    env_logger::init();

    let comm = Communication::connect()
        .await
        .context("Could not connect to the communication services")?;

    let app = App::new();
    let main_task = tokio::spawn(app.run(comm));

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
