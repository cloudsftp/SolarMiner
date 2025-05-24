use anyhow::{Context as AnyhowContext, Error, anyhow};
use async_nats::{Client, Message, Subject, jetstream::Context};
use bytes::Bytes;
use itertools::Itertools;
use log::debug;
use serde::Deserialize;

use crate::Config;

#[cfg(test)]
mod test;

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
struct PowerDemand {
    demand: usize,
    production: usize,
}

#[derive(Debug, PartialEq, Deserialize)]
enum PowerUpdateValue {
    #[serde(alias = "ON")]
    On,
    #[serde(alias = "OFF")]
    Off,
}

#[derive(Debug, PartialEq, Deserialize)]
enum CommandResult {
    #[serde(alias = "POWER")]
    Power(PowerUpdateValue),
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

#[derive(Debug, Deserialize)]
struct BatteryState {
    status: usize, // TODO: enum (4: discharge)
    state_of_charge: f32,
}

impl TryFrom<&Message> for UpdateEvent {
    type Error = Error;

    fn try_from(message: &Message) -> Result<Self, Self::Error> {
        let device_parts = message.subject.split(".").collect_vec();

        Ok(match device_parts.as_slice() {
            // Future: maybe use location
            ["stat", _location @ .., device, "RESULT"] => {
                let device = (*device).into();

                let result: CommandResult =
                    serde_json::from_slice(&message.payload).context(format!(
                        "could not decode payload '{}' received on subject '{}'",
                        String::from_utf8_lossy(&message.payload),
                        message.subject,
                    ))?;

                match result {
                    CommandResult::Power(value) => {
                        let on = matches!(value, PowerUpdateValue::On);
                        UpdateEvent::PlugStateUpdate { device, on }
                    }
                    CommandResult::EnergyConsumption {
                        total,
                        yesterday,
                        today,
                    } => {
                        todo!()
                    }
                }
            }
            ["stat", _location @ .., device, "POWER"] => {
                let plug_update: PowerUpdateValue = serde_plain::from_str(
                    &String::from_utf8_lossy(&message.payload),
                )
                .context(format!(
                    "could not decode payload '{}' received on subject '{}'",
                    String::from_utf8_lossy(&message.payload),
                    message.subject,
                ))?;

                let device = (*device).into();
                let on = matches!(plug_update, PowerUpdateValue::On);
                UpdateEvent::PlugStateUpdate { device, on }
            }
            ["solaredge", "powerflow"] => {
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
            ["solaredge", "modbus", "battery", _battery_name] => {
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
}
