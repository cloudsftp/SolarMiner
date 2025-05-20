use anyhow::{Context, Error, anyhow};
use async_nats::ConnectOptions;
use dotenv::dotenv;
use futures_util::StreamExt;
use log::{debug, error, info};
use nats_common::{
    MessageStream, connect_jetstream, connect_nats, create_stream, try_pub_sub_subscribe,
};
use std::{env, io, time::Duration};
use tokio::{
    signal::unix::{self, SignalKind},
    time::sleep,
};

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

async fn run(config: Config) -> Result<(), Error> {
    /*
    let js = connect_jetstream().await;
    let state_stream = create_stream(&js, &config.state_stream_name).await;

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

    let pi_host = env::var("PI_NATS_HOST")?;
    let pi_port = env::var("PI_NATS_PORT")?;
    let pi_options = ConnectOptions::new().token(env::var("PI_NATS_PASSWORD")?);

    let nats = pi_options
        .connect(format!("{}:{}", pi_host, pi_port))
        .await?;

    let mut plug_state_messages = nats.subscribe("stat.plug_bitaxe_001.LOGGING").await?;
    while let Some(message) = plug_state_messages.next().await {
        debug!("Received message {:?}", message);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
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

    Ok(())

    /*
    let mut mqttoptions = MqttOptions::new("Controller", env::var("MQTT_HOST")?, 1883);

    mqttoptions.set_credentials(env::var("MQTT_USER")?, env::var("MQTT_PASSWORD")?);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    let mut mqttoptions = MqttOptions::new(
        "Service",
        env::var("NATS_HOST").context("variable name: NATS_HOST")?,
        env::var("NATS_PORT")
            .context("variable name: NATS_PORT")?
            .parse()
            .context(format!("variable name: NATS_PORT, value: "))?,
    );

    if let Ok(token) = env::var("SERVER_TOKEN") {
        mqttoptions.set_credentials("", token);
    }

    let (service_client, service_eventloop) = AsyncClient::new(mqttoptions, 10);

    let client_sub = client.clone();
    tokio::spawn(async move {
        client_sub
            .subscribe("stat/+/RESULT", QoS::AtLeastOnce)
            .await
            .unwrap();
        println!("Subscribed to stat/plug_bitaxe_001/RESULT");

        State::run(eventloop).await;
    });

    loop {
        println!("press button to toggle plug");
        let _ = io::stdin().read_line(&mut String::new());

        client
            .publish(
                "cmnd/plug_bitaxe_001/POWER",
                QoS::AtLeastOnce,
                false,
                "TOGGLE",
            )
            .await?;
        println!("Published message.");
    }
     */
}
