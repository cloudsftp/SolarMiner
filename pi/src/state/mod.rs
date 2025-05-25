mod events;

#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::Message;
use events::UpdateEvent;
use log::debug;

use crate::Config;

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
    pub plug_state: PlugState,
    pub plug_energy: Option<EnergyState>,
    pub production_to_grid: Option<usize>,
    pub battery_level: Option<f32>,
}

impl State {
    pub async fn handle_message(
        mut self,
        config: &Config,
        message: &Message,
    ) -> Result<Self, Error> {
        // Update State
        let update = UpdateEvent::try_from(message)?;
        match update {
            UpdateEvent::PlugStateUpdate { device, on } => {
                if device != config.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.plug_state = if on { PlugState::On } else { PlugState::Off }
            }
            UpdateEvent::PlugEnergyUpdate {
                device,
                total,
                yesterday,
                today,
            } => {
                if device != config.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.plug_energy = Some(EnergyState {
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
                self.production_to_grid = Some(grid.production);
            }
            UpdateEvent::BatteryUpdate { level } => {
                self.battery_level = Some(level);
            }
            UpdateEvent::Unknown { subject, payload } => {
                debug!("Received message on subject '{}'", subject)
            }
        };

        debug!("Updated state: {:?}", self);

        Ok(self)
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
