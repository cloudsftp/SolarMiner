#[cfg(test)]
mod tests;

use anyhow::{Context as AnyhowContext, Error};
use async_nats::Message;
use serde::Deserialize;

use super::UpdateEvent;

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
}

#[derive(Debug, PartialEq, Deserialize)]
struct Status8 {
    #[serde(rename = "StatusSNS")]
    status_sns: StatusSNS,
}

#[derive(Debug, PartialEq, Deserialize)]
struct StatusSNS {
    #[serde(rename = "ENERGY")]
    energy: Status8Energy,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Status8Energy {
    total: f32,
    yesterday: f32,
    today: f32,
    power: f32,
}

pub fn decode_plug_message(topic_parts: &[&str], message: &Message) -> Result<UpdateEvent, Error> {
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
        [_location @ .., device, "STATUS8"] => {
            let status8: Status8 = serde_json::from_slice(&message.payload)?;
            let Status8Energy {
                total,
                yesterday,
                today,
                power,
            } = status8.status_sns.energy;

            UpdateEvent::PlugEnergyUpdate {
                device: device.to_string(),
                total,
                yesterday,
                today,
                power,
            }
        }
        _ => UpdateEvent::Unknown {
            subject: message.subject.clone(),
            payload: message.payload.clone(),
        },
    })
}
