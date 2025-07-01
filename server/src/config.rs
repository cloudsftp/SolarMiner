use std::env;

use anyhow::Error;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub state_stream_name: &'static str,
    pub commands_stream_name: &'static str,
}

pub fn load() -> Result<Config, Error> {
    let state_stream_name: &str = env::var("STATE_STREAM_NAME")?.leak();
    let commands_stream_name: &str = env::var("COMMANDS_STREAM_NAME")?.leak();

    Ok(Config {
        state_stream_name,
        commands_stream_name,
    })
}
