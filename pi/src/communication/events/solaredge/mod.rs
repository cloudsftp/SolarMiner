#[cfg(test)]
mod tests;

use anyhow::Error;
use async_nats::Message;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{PowerDemand, UpdateEvent};

#[derive(Debug, Deserialize)]
struct Power {
    pv_production: usize,
    grid: GridPower,
    battery: BatteryPower,
    consumer: ConsumerPower,
}

#[derive(Debug, Deserialize)]
struct GridPower {
    consumption: usize,
    delivery: usize,
}

#[derive(Debug, Deserialize)]
struct BatteryPower {
    charge: usize,
    discharge: usize,
}

#[derive(Debug, Deserialize)]
struct ConsumerPower {
    house: usize,
}

#[derive(Debug, PartialEq, Deserialize)]
struct BatteryState {
    status: BatteryStatus,
    state_of_charge: f32,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(usize)]
pub enum BatteryStatus {
    Off = 0,
    Standby = 1,
    Init = 2,
    Charging = 3,
    Discharging = 4,
    Fault = 5,
    PreservingCharge = 6,
    Idle = 7,
    PowerSaving = 10,
}

pub fn decode_solaredge_message(
    subject_parts: &[&str],
    message: &Message,
) -> Result<UpdateEvent, Error> {
    Ok(match subject_parts {
        ["powerflow"] => {
            let power_flow: Power = serde_json::from_slice(&message.payload)?;

            UpdateEvent::SolarPowerUpdate {
                pv_production: power_flow.pv_production as f32,
                house_demand: power_flow.consumer.house as f32,
                grid: PowerDemand {
                    demand: power_flow.grid.consumption as f32,
                    production: power_flow.grid.delivery as f32,
                },
                battery: PowerDemand {
                    demand: power_flow.battery.charge as f32,
                    production: power_flow.battery.discharge as f32,
                },
            }
        }
        ["modbus", "battery", _battery_name] => {
            let battery_state: BatteryState = serde_json::from_slice(&message.payload)?;

            UpdateEvent::BatteryUpdate {
                level: battery_state.state_of_charge,
            }
        }
        _ => UpdateEvent::Unknown {
            subject: message.subject.clone(),
            payload: message.payload.clone(),
        },
    })
}
