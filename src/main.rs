use anyhow::{Error, anyhow};
use dotenv::dotenv;
use log::{error, info};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use sitedata::{SiteDataProvider, solaredge::SolarEdgeDataProvider};
use std::{env, time::Duration};

mod sitedata;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;

    /*
        let api_key = env::var("API_KEY")?;
        let site_id = env::var("SITE_ID")?;

        let data_provider = SolarEdgeDataProvider::new(api_key, site_id);

        let excess_power = data_provider.get_current_excess_power().await;

        dbg!(excess_power);
    */

    /*
        let base_url = format!("{}{}/", API_URL, site_id);
        let url = format!("{}currentPowerFlow?api_key={}", base_url, api_key);
    */

    plug_example().await?;

    Ok(())
}

async fn plug_example() -> Result<(), Error> {
    let mut mqttoptions = MqttOptions::new("Pi", "192.168.0.47", 1883);

    mqttoptions.set_credentials(env::var("MQTT_USER")?, env::var("MQTT_PASSWORD")?);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

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
