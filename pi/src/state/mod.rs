mod update;

#[cfg(test)]
mod tests;

use std::time::Duration;

use crate::{CONFIG, Config, communication::Communication};
use anyhow::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlugState {
    On,
    Off,
    Unknown,
}

#[derive(Debug)]
pub struct EnergyState {
    total: f64,
    yesterday: f64,
    today: f64,
}

#[derive(Debug)]
pub struct State {
    plug: Plug,
    power: Option<PowerData>,
    battery_level: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct PowerData {
    from_grid: usize,
    from_pv: usize,
    to_house: usize,
    to_battery: usize,
    to_grid: usize,
}

#[derive(Debug)]
struct Plug {
    state: PlugState,
    energy: Option<EnergyState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            plug: Plug {
                state: PlugState::Unknown,
                energy: Default::default(),
            },
            power: Default::default(),
            battery_level: Default::default(),
        }
    }
}

impl State {
    pub fn mining_condition(&self) -> bool {
        match (self.battery_level, self.power) {
            (Some(level), _) if level > 30. => true,
            (
                Some(level),
                Some(PowerData {
                    from_pv, to_house, ..
                }),
            ) if level > 10. => {
                from_pv - to_house
                    > if matches!(self.plug.state, PlugState::On) {
                        0
                    } else {
                        CONFIG.controller.miner_demand
                    }
            }
            _ => false,
        }
    }

    pub async fn flip_plug_switch(&self, on: bool, comm: &Communication) -> Result<(), Error> {
        if self.send_plug_command_condition(on) {
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

    fn send_plug_command_condition(&self, on: bool) -> bool {
        matches!(
            (&self.plug.state, on),
            (PlugState::Unknown, _) | (PlugState::On, true) | (PlugState::Off, false)
        )
    }
}
