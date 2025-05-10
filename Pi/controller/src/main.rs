use anyhow::Error;
use dotenv::dotenv;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::{env, io, time::Duration};

mod plug;
mod state;

use state::State;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    dotenv()?;
    let mut mqttoptions = MqttOptions::new("Pi", env::var("MQTT_HOST")?, 1883);

    mqttoptions.set_credentials(env::var("MQTT_USER")?, env::var("MQTT_PASSWORD")?);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

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
}
