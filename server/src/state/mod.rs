use anyhow::Error;
use log::trace;

use crate::{
    communication::Communication,
    events::{StateUpdateEvent, StateUpdateEventMessage},
};

#[derive(Debug)]
pub struct State {
    //produced_energy: PowerIntegral,
    consumed_energy: PowerIntegral,
    //delivered_energy: PowerIntegral,
}

#[derive(Debug)]
struct PowerIntegral {}

impl PowerIntegral {
    fn new() -> Self {
        Self {}
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            consumed_energy: PowerIntegral::new(),
        }
    }

    async fn handle_state_update_event(
        &mut self,
        state_event: &StateUpdateEventMessage,
        comm: &Communication,
    ) -> Result<(), Error> {
        match state_event.event {
            StateUpdateEvent::MinerSwitch { on } => (),
            StateUpdateEvent::MinerPower {
                power,
                total_energy,
            } => (),
            StateUpdateEvent::HouseBattery { level } => (),
            StateUpdateEvent::HousePower {
                from_pv,
                from_battery,
                from_grid,
                to_house,
                to_battery,
                to_grid,
            } => {
                trace!("power: {} at {}", to_house, state_event.timestamp);
            }
        }

        trace!("updated state: {:?}", self);

        Ok(())
    }
}
