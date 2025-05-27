mod events;

use anyhow::{Error, anyhow};
use async_nats::Message;
use events::UpdateEvent;
use log::debug;

use crate::{
    App,
    state::{EnergyState, PlugState, PowerData},
};

impl App {
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

                self.state.plug.state = match on {
                    true => PlugState::On,
                    false => PlugState::Off,
                }
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

                self.state.plug.energy = Some(EnergyState {
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
                debug!(
                    "Received unexpected message on subject '{}' ({})",
                    subject,
                    String::from_utf8_lossy(&payload),
                )
            }
        };

        debug!("Updated state: {:?}", self.state);
        Ok(())
    }
}
