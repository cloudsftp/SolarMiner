mod events;

#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::{Client, Message, header::IntoHeaderName, jetstream::Context};
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
    config: Config,
    // TODO: comms (pi_nats, server_js)
    plug_state: PlugState,
    plug_energy: Option<EnergyState>,
    production_to_grid: Option<usize>,
    battery_level: Option<f32>,
}

impl State {
    pub async fn handle_message(
        mut self,
        message: Message,
        pi_nats: &Client,
        server_js: &Context,
    ) -> Result<Self, Error> {
        // Update State
        let update = UpdateEvent::try_from(&message)?;
        match update {
            UpdateEvent::PlugStateUpdate { device, on } => {
                if device != "plug_bitaxe_001" {
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
                if device != "plug_bitaxe_001" {
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

        // Perform Action

        let on = self
            .production_to_grid
            .is_some_and(|production| production > self.config.miner_demand);

        self.flip_plug_switch(&pi_nats, on).await?;

        Ok(self)
    }

    async fn flip_plug_switch(&self, pi_nats: &Client, on: bool) -> Result<(), Error> {
        if (on && self.plug_state == PlugState::On) || (!on && self.plug_state == PlugState::Off) {
            return Ok(());
        }

        let payload = if on { "ON" } else { "OFF" }.into();

        pi_nats
            .publish(format!("cmnd.{}.POWER", self.config.plug_name), payload)
            .await?;

        return Ok(());
    }
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            plug_state: PlugState::Unknown,
            plug_energy: None,
            battery_level: None,
            production_to_grid: None,
        }
    }
}
