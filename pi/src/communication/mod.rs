pub mod events;
mod initialization;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::{Client, Message, jetstream::Context};
use events::UpdateEvent;
use futures::{Stream, StreamExt, future::try_join_all, stream::select_all};
use once_cell::sync::Lazy;

use crate::{CONFIG, state::PartialState};

const PLUG_TOPICS: &[&str] = &["stat.*.RESULT", "stat.*.STATUS8"];
const SOLAREDGE_TOPICS: &[&str] = &["solaredge.modbus.battery.battery0", "solaredge.powerflow"];

static RELEVANT_SMART_HOME_TOPICS: Lazy<Vec<&str>> = Lazy::new(|| {
    let mut topics = Vec::new();
    topics.extend_from_slice(PLUG_TOPICS);
    topics.extend_from_slice(SOLAREDGE_TOPICS);
    topics
});

#[derive(Debug, Clone)]
pub struct Communication {
    pub pi_nats: Client,
    pub server_js: Context,
}

impl Communication {
    pub async fn get_update_events(
        &self,
    ) -> Result<impl Stream<Item = Result<UpdateEvent, Error>>, Error> {
        let pi_messages = self.subscribe_to_pi().await?;
        Ok(pi_messages.map(|message| UpdateEvent::try_from(&message)))
    }

    async fn subscribe_to_pi(&self) -> Result<impl Stream<Item = Message>, Error> {
        nats_subscribe(self.pi_nats.clone(), RELEVANT_SMART_HOME_TOPICS.to_vec())
            .await
            .context("Could not subscribe to the subjects on the controller")
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

    pub async fn report_state(&self, state: &PartialState) -> Result<(), Error> {
        let update_events = state.get_state_update_events();

        for event in update_events {
            self.server_js
                .publish(
                    CONFIG.communication.state_stream_name.clone(),
                    serde_json::to_vec(&event)
                        .context("could not serialize update event")?
                        .into(),
                )
                .await?;
        }

        Ok(())
    }
}

async fn nats_subscribe(
    nats: Client,
    subjects: Vec<&str>,
) -> Result<impl Stream<Item = Message>, Error> {
    let subscribers = try_join_all(
        subjects
            .iter()
            .map(async |subject| nats.subscribe(subject.to_string()).await),
    )
    .await?;

    Ok(select_all(subscribers))
}
