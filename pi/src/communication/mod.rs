use std::env;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::{
    Client, ConnectOptions,
    jetstream::{self, Context},
};

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
