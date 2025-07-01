mod initialization;

use anyhow::Error;
use async_nats::jetstream::{Context, Message, consumer::Consumer};
use futures::{Stream, StreamExt};

#[derive(Debug, Clone)]
pub struct Communication {
    js: Context,
}

#[derive(Debug, Clone)]
struct Streams {}

struct StateUpdateEvent {}

impl TryFrom<&Message> for StateUpdateEvent {
    type Error = Error;

    fn try_from(value: &Message) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Communication {
    /*
    pub async fn get_state_events(
        &self,
    ) -> Result<impl Stream<Item = Result<StateUpdateEvent, Error>>, Error> {
        let state_messages = self.subscribe_to_state_updates().await?;
        Ok(state_messages.map(|message| StateUpdateEvent::try_from(&message)))
    }

    async fn subscribe_to_state_updates(&self) -> Result<impl Stream<Item = Message>, Error> {
        self.state_consumer.messages().await
    }
     */
}
