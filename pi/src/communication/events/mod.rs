#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::Message;
use async_nats::Subject;
use bytes::Bytes;
use itertools::Itertools;
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, PartialEq)]
pub enum UpdateEvent {
    PlugStateUpdate {
        device: String,
        on: bool,
    },
    PlugEnergyUpdate {
        device: String,
        total: f64,
        yesterday: f64,
        today: f64,
    },
    PowerUpdate {
        pv_production: usize,
        house_demand: usize,
        grid: PowerDemand,
        battery: PowerDemand,
    },
    BatteryUpdate {
        level: f32,
    },
    Unknown {
        subject: Subject,
        payload: Bytes,
    },
}

#[derive(Debug, PartialEq)]
pub struct PowerDemand {
    pub demand: usize,
    pub production: usize,
}

// TODO: move structs and methods to decode messages to submodules

#[derive(Debug, PartialEq, Deserialize)]
enum PlugStateValue {
    #[serde(alias = "ON")]
    On,
    #[serde(alias = "OFF")]
    Off,
}

#[derive(Debug, PartialEq, Deserialize)]
enum CommandResult {
    #[serde(alias = "POWER")]
    Power(PlugStateValue),
    #[serde(
        alias = "EnergyTotal",
        alias = "EnergyYesterday",
        alias = "EnergyToday"
    )]
    EnergyConsumption {
        #[serde(alias = "Total")]
        total: f64,
        #[serde(alias = "Yesterday")]
        yesterday: f64,
        #[serde(alias = "Today")]
        today: f64,
    },
}

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
enum BatteryStatus {
    Standby = 1,
    Unknown2 = 2,
    Charging = 3,
    Discharging = 4,
    Fault = 5,
    PreservingCharge = 6,
    Idle = 7,
    Unknown8 = 8,
    Unknown9 = 9,
    PowerSaving = 10,
}

impl TryFrom<&Message> for UpdateEvent {
    type Error = Error;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        let subject_parts = message.subject.split(".").collect_vec();

        Ok(match subject_parts.as_slice() {
            ["stat", subject_parts @ ..] => decode_plug_message(subject_parts, message)?,
            ["solaredge", subject_parts @ ..] => decode_solaredge_message(subject_parts, message)?,
            _ => UpdateEvent::Unknown {
                subject: message.subject.clone(),
                payload: message.payload.clone(),
            },
        })
    }
}

fn decode_plug_message(topic_parts: &[&str], message: &Message) -> Result<UpdateEvent, Error> {
    Ok(match topic_parts {
        [_location @ .., device, "RESULT"] => {
            let device = (*device).into();

            let result: CommandResult =
                serde_json::from_slice(&message.payload).context(format!(
                    "could not decode payload '{}' received on subject '{}'",
                    String::from_utf8_lossy(&message.payload),
                    message.subject,
                ))?;

            match result {
                CommandResult::Power(value) => {
                    let on = matches!(value, PlugStateValue::On);
                    UpdateEvent::PlugStateUpdate { device, on }
                }
                CommandResult::EnergyConsumption {
                    total,
                    yesterday,
                    today,
                } => UpdateEvent::PlugEnergyUpdate {
                    device: device.to_string(),
                    total,
                    yesterday,
                    today,
                },
            }
        }
        [_location @ .., device, "POWER"] => {
            let plug_update: PlugStateValue = serde_plain::from_str(&String::from_utf8_lossy(
                &message.payload,
            ))
            .context(format!(
                "could not decode payload '{}' received on subject '{}'",
                String::from_utf8_lossy(&message.payload),
                message.subject,
            ))?;

            let device = (*device).into();
            let on = matches!(plug_update, PlugStateValue::On);
            UpdateEvent::PlugStateUpdate { device, on }
        }
        //[_location @ .., device, "STATUS8"] => {}
        _ => UpdateEvent::Unknown {
            subject: message.subject.clone(),
            payload: message.payload.clone(),
        },
    })
}

fn decode_solaredge_message(
    subject_parts: &[&str],
    message: &Message,
) -> Result<UpdateEvent, Error> {
    Ok(match subject_parts {
        ["powerflow"] => {
            let power_flow: Power = serde_json::from_slice(&message.payload)?;

            UpdateEvent::PowerUpdate {
                pv_production: power_flow.pv_production,
                house_demand: power_flow.consumer.house,
                grid: PowerDemand {
                    demand: power_flow.grid.consumption,
                    production: power_flow.grid.delivery,
                },
                battery: PowerDemand {
                    demand: power_flow.battery.charge,
                    production: power_flow.battery.discharge,
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
