mod initialization;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::{Client, jetstream::Context};

use crate::CONFIG;

#[derive(Debug, Clone)]
pub struct Communication {
    pub pi_nats: Client,
    pub server_js: Context,
}

impl Communication {
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
