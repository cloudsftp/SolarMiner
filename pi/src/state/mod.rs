mod events;

#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::{Client, Message, jetstream::Context};
use events::UpdateEvent;
use log::debug;

use crate::Config;

#[derive(Debug)]
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
}

impl State {
    pub async fn handle_message(
        mut self,
        message: Message,
        pi_nats: &Client,
        server_js: &Context,
    ) -> Result<Self, Error> {
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
                debug!("PV production: {}", pv_production);
                debug!("house demand: {}", house_demand);
            }
            UpdateEvent::BatteryUpdate { level } => {
                debug!("Battery level: {}", level)
            }
            UpdateEvent::Unknown { subject, payload } => {
                debug!("Received message on subject '{}'", subject)
            }
        };

        debug!("Updated state: {:?}", self);
        Ok(self)
    }
}

impl State {
    pub fn new(config: Config) -> Self {
        Self {
            plug_state: PlugState::Unknown,
            plug_energy: None,
        }
    }
}
