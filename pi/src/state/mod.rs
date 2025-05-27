mod action;
mod update;

#[cfg(test)]
mod tests;

use std::time::Duration;

use action::DampenedSwitch;

use crate::Config;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlugState {
    On,
    Off,
    Unknown,
}

#[derive(Debug)]
pub struct EnergyState {
    total: f64,
    yesterday: f64,
    today: f64,
}

#[derive(Debug)]
pub struct State {
    plug: Plug,
    power: Option<PowerData>,
    battery_level: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct PowerData {
    from_grid: usize,
    from_pv: usize,
    to_house: usize,
    to_battery: usize,
    to_grid: usize,
}

#[derive(Debug)]
struct Plug {
    state: PlugState,
    energy: Option<EnergyState>,
    switch: DampenedSwitch,
}

impl State {
    pub fn new(config: &Config) -> Self {
        Self {
            plug: Plug {
                state: PlugState::Unknown,
                energy: Default::default(),
                switch: DampenedSwitch::new(Duration::from_secs(
                    config.controller.switch_debounce_duration,
                )),
            },
            power: Default::default(),
            battery_level: Default::default(),
        }
    }
}
