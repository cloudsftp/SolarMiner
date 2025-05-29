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
    pub controller_commands_stream_name: String,
    pub plug_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ControllerConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub controller_interval: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub sensor_data_update_interval: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    pub switch_debounce_duration: Duration,
    pub miner_demand: usize,
    pub battery_low_threshold: f32,
    pub battery_high_threshold: f32,
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Self, Error> {
        let config_file =
            File::open(file_name).context(format!("Could not open config file '{}'", file_name))?;
        let config_file = BufReader::new(config_file);

        serde_yaml::from_reader(config_file)
            .context(format!("Could not parse config file '{}'", file_name))
    }
}
