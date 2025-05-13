use anyhow::{Context, Error, anyhow};
use itertools::Itertools;
use log::info;
use rumqttc::{Event, Packet, Publish};
use serde::Deserialize;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum PowerState {
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
    plug_power: PowerState,
    plug_energy: Option<EnergyState>,
}

impl State {
    pub async fn handle_event(mut self, event: Event) -> Result<Option<Self>, Error> {
        match event {
            Event::Incoming(Packet::Publish(publish)) => {
                let decoded = UpdateEvent::try_from(publish)?;
                info!("Received event: {:?}", decoded);

                match decoded {
                    UpdateEvent::PlugUpdate { device, on } => {
                        if device != "plug_bitaxe_001" {
                            return Err(anyhow!(
                                "received power update for unknown device '{}'",
                                device,
                            ));
                        }

                        self.plug_power = if on { PowerState::On } else { PowerState::Off }
                    }
                    UpdateEvent::EnergyUpdate {
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
                };
            }
            Event::Incoming(Packet::Disconnect) => {
                info!("Disconnected");
                return Ok(None);
            }
            _ => (),
        }

        info!("Updated state: {:?}", self);
        Ok(Some(self))
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            plug_power: PowerState::Unknown,
            plug_energy: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UpdateEvent {
    PlugUpdate {
        device: String,
        on: bool,
    },
    EnergyUpdate {
        device: String,
        total: f64,
        yesterday: f64,
        today: f64,
    },
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

impl TryFrom<Publish> for UpdateEvent {
    type Error = Error;

    fn try_from(value: Publish) -> Result<Self, Self::Error> {
        let device_parts = value.topic.split("/").collect_vec();

        Ok(match device_parts.as_slice() {
            // Future: maybe use location
            ["stat", _location @ .., device, "RESULT"] => {
                let device = (*device).into();

                let result: CommandResult =
                    serde_json::from_slice(&value.payload).context(format!(
                        "could not decode payload '{}' received on topic '{}'",
                        String::from_utf8_lossy(&value.payload),
                        value.topic,
                    ))?;

                match result {
                    CommandResult::Power(value) => {
                        let on = matches!(value, PowerUpdateValue::On);
                        UpdateEvent::PlugUpdate { device, on }
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
                    &String::from_utf8_lossy(&value.payload),
                )
                .context(format!(
                    "could not decode payload '{}' received on topic '{}'",
                    String::from_utf8_lossy(&value.payload),
                    value.topic,
                ))?;

                let device = (*device).into();
                let on = matches!(plug_update, PowerUpdateValue::On);
                UpdateEvent::PlugUpdate { device, on }
            }
            _ => todo!(),
        })
    }
}
