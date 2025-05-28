use std::env;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions, Message,
    jetstream::{self, Context},
};
use futures::{Stream, future::try_join_all, stream::select_all};

use crate::CONFIG;

#[derive(Debug, Clone)]
pub struct Communication {
    pub pi_nats: Client,
    pub server_js: Context,
}

impl Communication {
    pub async fn connect() -> Result<Self, Error> {
        let pi_nats = connect_nats_client("PI").await?;

        let server_nats = connect_nats_client("SERVER").await?;
        let server_js = jetstream::new(server_nats);

        Ok(Self { pi_nats, server_js })
    }

    pub async fn query_plug_state(&self) -> Result<(), Error> {
        self.pi_nats
            .publish(
                format!("cmnd.{}.Power", CONFIG.communication.plug_name),
                "".into(),
            )
            .await
            .context("Could not query switch state of plug")?;

        self.pi_nats
            .publish(
                format!("cmnd.{}.Status", CONFIG.communication.plug_name),
                "8".into(),
            )
            .await
            .context("Could not query power state of plug")?;

        Ok(())
    }

    pub async fn flip_plug_switch(&self, on: bool) -> Result<(), Error> {
        let payload = if on { "ON" } else { "OFF" }.into();

        self.pi_nats
            .publish(
                format!("cmnd.{}.POWER", CONFIG.communication.plug_name),
                payload,
            )
            .await?;

        Ok(())
    }
}

pub async fn nats_subscribe(
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
