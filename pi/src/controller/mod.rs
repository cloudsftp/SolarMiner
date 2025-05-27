mod switch;

use anyhow::Error;
use switch::DampenedSwitch;

use crate::{CONFIG, communication::Communication, state::State};

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

impl Controller {
    pub async fn perform_action(
        &mut self,
        state: &State,
        comm: &Communication,
    ) -> Result<(), Error> {
        let on = state.mining_condition();

        if self.switch.perform(on) {
            self.flip_plug_switch(on, state, comm).await?;
        }

        Ok(())
    }

    async fn flip_plug_switch(
        &self,
        on: bool,
        state: &State,
        comm: &Communication,
    ) -> Result<(), Error> {
        if state.skip_plug_command_condition(on) {
            return Ok(());
        }

        let payload = if on { "ON" } else { "OFF" }.into();

        comm.pi_nats
            .publish(
                format!("cmnd.{}.POWER", CONFIG.communication.plug_name),
                payload,
            )
            .await?;

        Ok(())
    }
}
