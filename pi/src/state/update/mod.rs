use anyhow::{Error, anyhow};
use async_nats::Message;
use log::debug;

use crate::{
    CONFIG,
    communication::events::UpdateEvent,
    state::{EnergyState, PlugState, PowerData, State},
};

impl State {
    pub async fn handle_message(&mut self, message: &Message) -> Result<(), Error> {
        let update = UpdateEvent::try_from(message)?;
        match update {
            UpdateEvent::PlugStateUpdate { device, on } => {
                if device != CONFIG.communication.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.plug.state = match on {
                    true => PlugState::On,
                    false => PlugState::Off,
                }
            }
            UpdateEvent::PlugEnergyUpdate {
                device,
                total,
                yesterday,
                today,
                power,
            } => {
                if device != CONFIG.communication.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.plug.energy = Some(EnergyState {
                    total,
                    yesterday,
                    today,
                });
            }
            UpdateEvent::SolarPowerUpdate {
                pv_production,
                house_demand,
                grid,
                battery,
            } => {
                self.power = Some(PowerData {
                    from_pv: pv_production,
                    from_battery: battery.production,
                    from_grid: grid.demand,
                    to_house: house_demand,
                    to_battery: battery.demand,
                    to_grid: grid.production,
                });
            }
            UpdateEvent::BatteryUpdate { level } => {
                self.battery_level = Some(level);
            }
            UpdateEvent::Unknown { subject, payload } => {
                debug!(
                    "Received unexpected message on subject '{}' ({})",
                    subject,
                    String::from_utf8_lossy(&payload),
                )
            }
        };

        debug!("Updated state: {:?}", self);
        Ok(())
    }
}
