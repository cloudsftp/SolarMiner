use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions, Message,
    jetstream::{self, Context},
};
use dotenv::dotenv;
use futures::{Stream, StreamExt, future::try_join_all, stream::select_all};
use log::{debug, error, info};
use std::env;
use tokio::signal::unix::{self, SignalKind};

mod plug;
mod state;

#[derive(Debug, Clone, Copy)]
struct Config {
    state_stream_name: &'static str,
    controller_commands_stream_name: &'static str,
}

use state::State;

/*
impl State {
    async fn run(mut eventloop: EventLoop) {
        let mut state = Self::default();

        loop {
            let event = eventloop.poll().await.expect("for now, panic"); // TODO: remove panic
            state = match state.handle_event(event).await.expect("for now, panic") {
                Some(state) => state,
                None => break,
            };
        }
    }
}
*/

async fn run(config: Config, pi_nats: Client, server_js: Context) -> Result<(), Error> {
    /*
    let controller_command_stream =
        create_stream(&js, &config.controller_commands_stream_name).await;
    let mut command_messages: MessageStream =
        try_pub_sub_subscribe(&js, &config.controller_commands_stream_name)
            .await
            .map_err(|err| anyhow!(err)) // TODO: remove as soon as library has better errors
            .context("could not subscribe to controller state stream")?;


    while let Some(message) = plug_state_messages.next().await {
        debug!("Received message {:?}", message);
    }
    */

    // TODO: multiple subscribers: https://natsbyexample.com/examples/messaging/iterating-multiple-subscriptions/rust
    let mut pi_messages = nats_subscribe(&pi_nats, &["stat.*.RESULT"]).await?;

    info!("subscribed to nats");

    let mut state = State::default();
    while let Some(message) = pi_messages.next().await {
        debug!("Received message {:?}", message);
        info!("Topic: {}", message.subject.to_string());
        state = state.handle_message(message).await?;
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

    let state_stream_name: &str = get_env("STATE_STREAM_NAME")?.leak();
    let controller_commands_stream_name: &str = get_env("CONTROLLER_COMMANDS_STREAM_NAME")?.leak();

    let config = Config {
        state_stream_name,
        controller_commands_stream_name,
    };

    let pi_nats = connect_nats_client("PI").await?;

    let server_nats = connect_nats_client("SERVER").await?;
    let server_js = jetstream::new(server_nats);

    let main_task = tokio::spawn(run(config, pi_nats, server_js)); // TODO: wrap communications in struct, extra thread for sending?

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
    tokio::spawn(async move {
        client_sub
            .subscribe("stat/+/RESULT", QoS::AtLeastOnce)
            .await
            .unwrap();
        println!("Subscribed to stat/plug_bitaxe_001/RESULT");

        State::run(eventloop).await;
    });

     */

    Ok(())
}

async fn connect_nats_client(prefix: &str) -> Result<Client, Error> {
    let get_env = |name| get_env(&format!("{}_{}", prefix, name));

    let host = get_env("NATS_HOST")?;
    let port = get_env("NATS_PORT")?;
    let options = ConnectOptions::new().token(get_env("NATS_PASSWORD")?);

    options
        .connect(format!("{}:{}", host, port))
        .await
        .context(format!("Could not connect to nats server '{}'", prefix))
}

fn get_env(key: &str) -> Result<String, Error> {
    env::var(key).context(format!("could not get value for key '{}'", key))
}
