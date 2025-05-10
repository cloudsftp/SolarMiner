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
pub struct State {
    plug: PowerState,
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
                                device
                            ));
                        }

                        self.plug = if on { PowerState::On } else { PowerState::Off }
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
            plug: PowerState::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateEvent {
    PlugUpdate { device: String, on: bool },
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum PowerUpdateValue {
    #[serde(alias = "ON")]
    On,
    #[serde(alias = "OFF")]
    Off,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum CommandResult {
    #[serde(alias = "POWER")]
    Power(PowerUpdateValue),
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
