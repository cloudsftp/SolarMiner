mod plug;
mod solaredge;

use anyhow::Error;
use async_nats::Message;
use async_nats::Subject;
use bytes::Bytes;
use itertools::Itertools;
use plug::decode_plug_message;
use solaredge::decode_solaredge_message;

#[derive(Debug, PartialEq)]
pub enum UpdateEvent {
    PlugStateUpdate {
        device: String,
        on: bool,
    },
    PlugEnergyUpdate {
        device: String,
        total: f32,
        yesterday: f32,
        today: f32,
        power: f32,
    },
    SolarPowerUpdate {
        pv_production: f32,
        house_demand: f32,
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
    pub demand: f32,
    pub production: f32,
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
