use anyhow::Error;
use dotenv::dotenv;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::{env, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;
    let mut mqttoptions = MqttOptions::new("Pi", env::var("MQTT_HOST")?, 1883);

    mqttoptions.set_credentials(env::var("MQTT_USER")?, env::var("MQTT_PASSWORD")?);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let client_sub = client.clone();
    tokio::spawn(async move {
        client_sub
            .subscribe("stat/plug_bitaxe_001/RESULT", QoS::AtLeastOnce)
            .await
            .unwrap();
        println!("Subscribed to stat/plug_bitaxe_001/RESULT");

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    println!(
                        "Received = '{}' on topic '{}'",
                        String::from_utf8_lossy(&publish.payload),
                        publish.topic
                    );
                }
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    println!("Disconnected");
                    break;
                }
                Err(e) => {
                    eprintln!("Error in eventloop: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {}
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    client
        .publish(
            "cmnd/plug_bitaxe_001/Power",
            QoS::AtLeastOnce,
            false,
            "TOGGLE",
        )
        .await?;
    println!("Published message.");

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}
