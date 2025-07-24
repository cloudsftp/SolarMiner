mod switch;

use anyhow::Error;
use log::debug;
use switch::DampenedSwitch;

use crate::{CONFIG, communication::Communication, state::PartialState};

#[derive(Debug)]
pub struct Controller {
    switch: DampenedSwitch,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            switch: DampenedSwitch::new(CONFIG.controller.switch.debounce_duration),
        }
    }
}

impl Controller {
    pub async fn perform_action(
        &mut self,
        state: &PartialState,
        comm: &Communication,
    ) -> Result<(), Error> {
        debug!("Controller performing action");
        let on = state.mining_condition();

        if self.switch.perform(on) && !state.should_skip_send_plug_command(on) {
            comm.flip_plug_switch(on).await?;
        }

        Ok(())
    }
}
