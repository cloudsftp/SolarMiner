use anyhow::{Error, anyhow};
use log::debug;

use crate::{
    CONFIG,
    communication::events::UpdateEvent,
    state::{EnergyState, PartialState, PowerData},
};

impl PartialState {
    pub async fn update(&mut self, update: Result<UpdateEvent, Error>) -> Result<(), Error> {
        match update? {
            UpdateEvent::PlugStateUpdate { device, on } => {
                if device != CONFIG.communication.plug_name {
                    return Err(anyhow!(
                        "received power update for unknown device '{}'",
                        device,
                    ));
                }

                self.plug.on.set(on)
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

                self.plug.energy.set(EnergyState {
                    total,
                    yesterday,
                    today,
                    power,
                });
            }
            UpdateEvent::SolarPowerUpdate {
                pv_production,
                house_demand,
                grid,
                battery,
            } => {
                self.inverter.power.set(PowerData {
                    from_pv: pv_production,
                    from_battery: battery.production,
                    from_grid: grid.demand,
                    to_house: house_demand,
                    to_battery: battery.demand,
                    to_grid: grid.production,
                });
            }
            UpdateEvent::BatteryUpdate { level } => {
                self.inverter.battery_level.set(level);
            }
            UpdateEvent::Unknown { subject, payload } => {
                debug!(
                    "Received unexpected message on subject '{}' ({})",
                    subject,
                    String::from_utf8_lossy(&payload),
                )
            }
        };

        debug!("Updated state: {self:?}");
        Ok(())
    }
}
