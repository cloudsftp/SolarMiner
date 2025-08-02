use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions,
    jetstream::{self, Context, consumer, stream},
};
use std::{env, time::Duration};

use crate::CONFIG;

use super::Communication;

const SECONDS_IN_A_DAY: u64 = 24 * 60 * 60;
const SECONDS_IN_A_WEEK: u64 = 7 * SECONDS_IN_A_DAY;

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub state_stream_info: stream::Info,
    pub commands_stream_info: stream::Info,
}

impl Communication {
    pub async fn connect() -> Result<Self, Error> {
        let server_nats = connect_nats_client("SERVER").await?;
        let js = jetstream::new(server_nats);

        create_service_streams(&js).await?;
        let state_stream_consumer = js
            .create_consumer_on_stream(
                consumer::pull::Config {
                    durable_name: Some("service".to_string()),
                    ..Default::default()
                },
                &CONFIG.state_stream_name,
            )
            .await?;

        Ok(Self {
            js,
            state_stream_consumer,
        })
    }
}

async fn create_service_streams(js: &Context) -> Result<(), Error> {
    js.create_or_update_stream(stream::Config {
        name: CONFIG.state_stream_name.to_string(),
        discard: stream::DiscardPolicy::Old,
        max_age: Duration::from_secs(SECONDS_IN_A_WEEK),
        ..Default::default()
    })
    .await
    .context("Could not create the state stream for the service")?;

    js.create_or_update_stream(stream::Config {
        name: CONFIG.commands_stream_name.to_string(),
        discard: stream::DiscardPolicy::Old,
        max_age: Duration::from_secs(SECONDS_IN_A_WEEK),
        ..Default::default()
    })
    .await
    .context("Could not create the commands stream for the service")?;

    Ok(())
}
async fn connect_nats_client(prefix: &str) -> Result<Client, Error> {
    let host = env::var("NATS_HOST").context("reading NATS_HOST")?;
    let port = env::var("NATS_PORT").context("reading NATS_PORT")?;
    let options =
        ConnectOptions::new().token(env::var("NATS_TOKEN").context("reading NATS_TOKEN")?);

    options
        .connect(format!("{host}:{port}"))
        .await
        .context(format!("Could not connect to nats server '{prefix}'"))
}
