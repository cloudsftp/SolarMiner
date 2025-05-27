use anyhow::{Context as AnyhowContext, Error};
use serde::Deserialize;
use std::{fs::File, io::BufReader};

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
    pub controller_time: f32,
    pub miner_demand: usize,
    pub switch_debounce_duration: f32,
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
