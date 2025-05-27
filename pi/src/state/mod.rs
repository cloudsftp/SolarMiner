mod events;

#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::Message;
use events::UpdateEvent;
use log::debug;

use crate::App;

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
    plug_state: PlugState,
    plug_energy: Option<EnergyState>,
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

impl App {
    pub fn mining_condition(&self) -> bool {
        match (self.state.battery_level, self.state.power) {
            (Some(level), _) if level > 30. => true,
            (
                Some(level),
                Some(PowerData {
                    from_pv, to_house, ..
                }),
            ) if level > 10. => {
                from_pv - to_house
                    > if matches!(self.state.plug_state, PlugState::On) {
                        0
                    } else {
                        self.config.miner_demand
                    }
            }
            _ => false,
        }
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
                self.state.power = Some(PowerData {
                    from_grid: grid.demand,
                    from_pv: pv_production,
                    to_house: house_demand,
                    to_battery: battery.demand,
                    to_grid: grid.production,
                });
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
            power: Default::default(),
            battery_level: Default::default(),
        }
    }
}
