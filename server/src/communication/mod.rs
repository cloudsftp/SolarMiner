mod initialization;

use anyhow::Error;
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

#[derive(Debug, Clone)]
struct Streams {}

impl TryFrom<Message> for StateUpdateEvent {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        todo!()
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
