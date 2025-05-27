use std::time::{Duration, Instant};

use anyhow::Error;

use crate::{App, config};

use super::{PlugState, PowerData};

impl App {
    pub async fn perform_control_action(&mut self) -> Result<(), Error> {
        let on = self.mining_condition();
        self.flip_plug_switch(on).await
    }

    fn mining_condition(&self) -> bool {
        match (self.state.battery_level, self.state.power) {
            (Some(level), _) if level > 30. => true,
            (
                Some(level),
                Some(PowerData {
                    from_pv, to_house, ..
                }),
            ) if level > 10. => {
                from_pv - to_house
                    > if matches!(self.state.plug.state, PlugState::On) {
                        0
                    } else {
                        config.controller.miner_demand
                    }
            }
            _ => false,
        }
    }

    async fn flip_plug_switch(&mut self, on: bool) -> Result<(), Error> {
        if !self.state.plug.switch.perform(on) || self.send_plug_command_condition(on) {
            return Ok(());
        }

        let payload = if on { "ON" } else { "OFF" }.into();

        self.comm
            .pi_nats
            .publish(
                format!("cmnd.{}.POWER", config.communication.plug_name),
                payload,
            )
            .await?;

        Ok(())
    }

    fn send_plug_command_condition(&self, on: bool) -> bool {
        matches!(
            (&self.state.plug.state, on),
            (PlugState::Unknown, _) | (PlugState::On, true) | (PlugState::Off, false)
        )
    }
}

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
