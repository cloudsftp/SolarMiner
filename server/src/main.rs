use anyhow::{Context, Error};
use config::Config;
use dotenv::dotenv;
use futures::StreamExt;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use tokio::signal::unix::{self, SignalKind};

mod communication;
mod config;
mod events;

use communication::Communication;

#[derive(Debug)]
struct App {
    //house_state: PartialState,
}

static CONFIG: Lazy<Config> = Lazy::new(|| config::load().expect("could not load config"));

impl App {
    async fn run(mut self, comm: Communication) -> Result<(), Error> {
        debug!("hello");
        let comm = Communication::connect().await?;

        let mut state_events = comm.get_state_events().await?;

        while let Some(state_event) = state_events.next().await {
            debug!("hell yeah");
            dbg!(state_event);
        }

        Ok(())
    }
}

impl App {
    fn new() -> Self {
        App {
    //        state: PartialState::new(),
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
