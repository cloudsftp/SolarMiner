mod switch;

#[derive(Debug)]
pub struct Controller {
    switch: DampenedSwitch,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            switch: DampenedSwitch::new(CONFIG.controller.switch_debounce_duration),
        }
    }
}

use anyhow::Error;
use switch::DampenedSwitch;

use crate::{CONFIG, communication::Communication, state::State};

impl Controller {
    pub async fn perform_action(
        &mut self,
        state: &State,
        comm: &Communication,
    ) -> Result<(), Error> {
        let on = state.mining_condition();

        if self.switch.perform(on) {
            state.flip_plug_switch(on, &comm).await?;
        }

        Ok(())
    }
}
