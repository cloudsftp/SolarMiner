use anyhow::{Context, Error};
use itertools::Itertools;
use rumqttc::Publish;
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

#[derive(Debug, Deserialize)]
enum PlugUpdateValue {
    #[serde(alias = "ON")]
    On,
    #[serde(alias = "OFF")]
    Off,
}

#[derive(Debug, Deserialize)]
struct PlugUpdate {
    #[serde(alias = "POWER")]
    power: PlugUpdateValue,
}

impl TryFrom<Publish> for UpdateEvent {
    type Error = Error;

    fn try_from(value: Publish) -> Result<Self, Self::Error> {
        let device_parts = value.topic.split("/").collect_vec();

        Ok(match device_parts.as_slice() {
            ["stat", device, "POWER"] => {
                let plug_update: PlugUpdate =
                    serde_json::from_slice(&value.payload).context(format!(
                        "could not decode payload '{}' received on topic '{}'",
                        String::from_utf8_lossy(&value.payload),
                        value.topic,
                    ))?;

                let device = (*device).into();
                let on = matches!(plug_update.power, PlugUpdateValue::On);
                UpdateEvent::PlugUpdate { device, on }
            }
            _ => todo!(),
        })
    }
}
