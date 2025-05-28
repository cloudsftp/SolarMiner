use std::env;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions, Message,
    jetstream::{
        self,
        stream::{self, Info},
    },
};
use futures::{Stream, future::try_join_all, stream::select_all};

use crate::CONFIG;

use super::Communication;

impl Communication {
    pub async fn connect() -> Result<Self, Error> {
        let pi_nats = connect_nats_client("PI").await?;

        let server_nats = connect_nats_client("SERVER").await?;
        let server_js = jetstream::new(server_nats);

        Ok(Self { pi_nats, server_js })
    }

    pub async fn create_service_streams(&self) -> Result<Info, Error> {
        self.server_js
            .create_or_update_stream(stream::Config {
                name: CONFIG.communication.state_stream_name.clone(),
                ..Default::default()
            })
            .await
            .context("Could not create the state stream for the service")
    }

    pub async fn subscribe_to_pi(&self) -> Result<impl Stream<Item = Message>, Error> {
        nats_subscribe(
            self.pi_nats.clone(),
            &[
                "stat.*.RESULT",
                "stat.*.STATUS8",
                "solaredge.modbus.battery.battery0",
                "solaredge.powerflow",
            ],
        )
        .await
        .context("Could not subscribe to the subjects on the controller")
    }
}

async fn nats_subscribe(
    nats: Client,
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

async fn connect_nats_client(prefix: &str) -> Result<Client, Error> {
    let get_env = |name| get_env(&format!("{}_{}", prefix, name));

    let host = get_env("NATS_HOST")?;
    let port = get_env("NATS_PORT")?;
    let options = ConnectOptions::new().token(get_env("NATS_TOKEN")?);

    options
        .connect(format!("{}:{}", host, port))
        .await
        .context(format!("Could not connect to nats server '{}'", prefix))
}

fn get_env(key: &str) -> Result<String, Error> {
    env::var(key).context(format!("could not get value for key '{}'", key))
}
