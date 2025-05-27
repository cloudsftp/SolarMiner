#[derive(Debug)]
pub struct Controller {}

impl Controller {
    pub fn new() -> Self {
        Controller {}
    }
}

use anyhow::Error;

use crate::{communication::Communication, state::State};

impl Controller {
    pub async fn perform_action(
        &mut self,
        state: &State,
        comm: &Communication,
    ) -> Result<(), Error> {
        let on = state.mining_condition();
        state.flip_plug_switch(on, &comm).await
    }
}

mod switch {
    use std::time::{Duration, Instant};

    #[derive(Debug)]
    pub struct DampenedSwitch {
        time_to_switch: Duration,
        command: bool,
        received_since: Instant,
    }

    impl DampenedSwitch {
        pub fn new(time_to_switch: Duration) -> Self {
            Self {
                time_to_switch,
                command: false,
                received_since: Instant::now(),
            }
        }

        fn perform(&mut self, command: bool) -> bool {
            if command != self.command {
                self.command = command;
                self.received_since = Instant::now();
                false
            } else {
                Instant::now().duration_since(self.received_since) > self.time_to_switch
            }
        }
    }
}
