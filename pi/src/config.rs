use anyhow::{Context as AnyhowContext, Error};
use duration_str::deserialize_duration;
use serde::Deserialize;
use std::{fs::File, io::BufReader, time::Duration};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub communication: CommunicationConfig,
    pub controller: ControllerConfig,
}

#[derive(Debug, Deserialize)]
pub struct CommunicationConfig {
    pub state_stream_name: String,
    pub commands_stream_name: String,
    pub plug_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ControllerConfig {
    pub controller: Controller,
    pub sensor_data: SensorData,
    pub switch: Switch,
    pub miner: Miner,
    pub battery: Battery,
}

#[derive(Debug, Deserialize)]
pub struct Controller {
    #[serde(deserialize_with = "deserialize_duration")]
    pub action_interval: Duration,
}

#[derive(Debug, Deserialize)]
pub struct SensorData {
    #[serde(deserialize_with = "deserialize_duration")]
    pub update_interval: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub control_duration: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Switch {
    #[serde(deserialize_with = "deserialize_duration")]
    pub debounce_duration: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Miner {
    pub demand: f32,
}

#[derive(Debug, Deserialize)]
pub struct Battery {
    pub low_threshold: f32,
    pub high_threshold: f32,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        let config_file =
            File::open(file_name).context(format!("Could not open config file '{file_name}'"))?;
        let config_file = BufReader::new(config_file);

        serde_yaml::from_reader(config_file)
            .context(format!("Could not parse config file '{file_name}'"))
    }
}
