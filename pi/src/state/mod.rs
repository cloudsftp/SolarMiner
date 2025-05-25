mod events;

#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::Message;
use events::UpdateEvent;
use log::debug;

use crate::App;

#[derive(Debug, PartialEq)]
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
    plug_state: PlugState,
    plug_energy: Option<EnergyState>,
    production_to_grid: Option<usize>,
    battery_level: Option<f32>,
}

impl App {
    pub fn mining_condition(&self) -> bool {
        self.state
            .production_to_grid
            .is_some_and(|production| production > self.config.miner_demand)
    }

    pub fn plug_state_satisfied(&self, on: bool) -> bool {
        matches!(
            (&self.state.plug_state, on),
            (PlugState::On, true) | (PlugState::Off, false)
        )
    }

    pub async fn update_state(&mut self, message: &Message) -> Result<(), Error> {
        let update = UpdateEvent::try_from(message)?;
        match update {
            UpdateEvent::PlugStateUpdate { device, on } => {
                if device != self.config.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.state.plug_state = if on { PlugState::On } else { PlugState::Off }
            }
            UpdateEvent::PlugEnergyUpdate {
                device,
                total,
                yesterday,
                today,
            } => {
                if device != self.config.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.state.plug_energy = Some(EnergyState {
                    total,
                    yesterday,
                    today,
                })
            }
            UpdateEvent::PowerUpdate {
                pv_production,
                house_demand,
                grid,
                battery,
            } => {
                self.state.production_to_grid = Some(grid.production);
            }
            UpdateEvent::BatteryUpdate { level } => {
                self.state.battery_level = Some(level);
            }
            UpdateEvent::Unknown { subject, payload } => {
                debug!("Received message on subject '{}'", subject)
            }
        };

        debug!("Updated state: {:?}", self.state);
        Ok(())
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            plug_state: PlugState::Unknown,
            plug_energy: Default::default(),
            production_to_grid: Default::default(),
            battery_level: Default::default(),
        }
    }
}
