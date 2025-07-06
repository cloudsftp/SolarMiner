mod initialization;

use anyhow::{Context as _, Error};
use async_nats::jetstream::{
    Context, Message,
    consumer::{Consumer, pull::Config},
};
use futures::TryStreamExt;
use futures_util::{Stream, StreamExt};

use crate::events::StateUpdateEvent;

#[derive(Debug, Clone)]
pub struct Communication {
    js: Context,
    state_stream_consumer: Consumer<Config>,
}

impl TryFrom<Message> for StateUpdateEvent {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.payload).context("could not decode update event")
    }
}

impl Communication {
    pub async fn get_state_events(
        &self,
    ) -> Result<impl Stream<Item = Result<StateUpdateEvent, Error>>, Error> {
        Ok(self
            .state_stream_consumer
            .messages()
            .await?
            .map_err(Error::from)
            .map(|message| message.and_then(StateUpdateEvent::try_from)))
    }
}
