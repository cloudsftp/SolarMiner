mod communication;
mod config;
mod state;

use anyhow::{Context, Error};
use config::Config;
use dotenv::dotenv;
use futures::StreamExt;
use log::{error, info, trace};
use once_cell::sync::Lazy;
use solarminer_service::events::StateUpdateEventMessage;
use state::State;
use tokio::signal::unix::{self, SignalKind};

use communication::Communication;

#[derive(Debug)]
struct App {
    state: State,
}

static CONFIG: Lazy<Config> = Lazy::new(|| config::load().expect("could not load config"));

impl App {
    async fn run(self, comm: Communication) -> Result<(), Error> {
        let mut state_events = comm.get_state_events().await?;

        while let Some(state_event) = state_events.next().await {
            info!("received event {:?}", state_event);

            let state_event = match state_event {
                Ok(state_event) => state_event,
                Err(err) => {
                    error!("while receiving message from state event stream: {}", err);
                    continue;
                }
            };

            self.state
                .handle_update_state_event(&state_event, &comm)
                .await?;
        }

        Ok(())
    }
}

impl App {
    fn new() -> Self {
        App {
            state: State::new(),
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
                    error!("Main task errored: {err}")
                },
                Err(err) => {
                    error!("Could not join main task: {err}")
                },
            }
        }
    }

    Ok(())
}
