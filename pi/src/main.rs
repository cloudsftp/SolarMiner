use anyhow::{Context, Error};
use dotenv::dotenv;
use std::{env, io, time::Duration};

mod plug;
mod state;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv()?;

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

    let main_task = tokio::spawn(run(config));


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
