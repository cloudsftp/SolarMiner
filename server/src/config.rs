use std::env;

use anyhow::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub state_stream_name: String,
    pub commands_stream_name: String,
}

pub fn load() -> Result<Config, Error> {
    let state_stream_name = env::var("STATE_STREAM_NAME")?;
    let commands_stream_name = env::var("COMMANDS_STREAM_NAME")?;

    Ok(Config {
        state_stream_name,
        commands_stream_name,
    })
}
