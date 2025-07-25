mod initialization;

use anyhow::{Context as _, Error};
use async_nats::jetstream::{
    Context, Message,
    consumer::{Consumer, pull::Config},
};
use futures::TryStreamExt;
use futures_util::{Stream, StreamExt};
use log::{error, trace};

use crate::events::StateUpdateEventMessage;

#[derive(Debug, Clone)]
pub struct Communication {
    js: Context,
    state_stream_consumer: Consumer<Config>,
}

impl TryFrom<Message> for StateUpdateEventMessage {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        serde_json::from_slice(&value.payload).context("could not decode update event")
    }
}

impl Communication {
    pub async fn get_state_events(
        &self,
    ) -> Result<impl Stream<Item = Result<StateUpdateEventMessage, Error>>, Error> {
        Ok(self
            .state_stream_consumer
            .messages()
            .await?
            .map_err(Error::from)
            .map(|message| {
                message
                    .inspect_err(|err| {
                        error!(
                            "could not receive message from controller state stream: {}",
                            err,
                        )
                    })
                    .inspect(|message| {
                        trace!(
                            "received message from controller state stream:\n{:?}",
                            message,
                        );
                    })
                    .and_then(StateUpdateEventMessage::try_from)
            }))
    }
}
