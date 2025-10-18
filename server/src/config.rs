use std::env;

use anyhow::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub state_stream_name: String,
    pub commands_stream_name: String,
    pub aggregation_kv_name: String,
    pub development: bool,
}

pub fn load() -> Result<Config, Error> {
    let state_stream_name = env::var("STATE_STREAM_NAME")?;
    let commands_stream_name = env::var("COMMANDS_STREAM_NAME")?;
    let aggregation_kv_name = env::var("AGGREGATION_KV_NAME")?;
    let development = env::var("DEV").is_ok_and(|dev| dev == "true");

    Ok(Config {
        state_stream_name,
        commands_stream_name,
        aggregation_kv_name,
        development,
    })
}
