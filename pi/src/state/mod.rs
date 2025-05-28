mod update;

#[cfg(test)]
mod tests;

use crate::CONFIG;

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
}

impl State {
    pub fn new() -> Self {
        Self {
            plug: Plug {
                state: PlugState::Unknown,
                energy: Default::default(),
            },
            power: Default::default(),
            battery_level: Default::default(),
        }
    }
}

impl State {
    pub fn mining_condition(&self) -> bool {
        match (self.battery_level, self.power) {
            (Some(level), _) if level > CONFIG.controller.battery_high_threshold => true,
            (
                Some(level),
                Some(PowerData {
                    from_pv, to_house, ..
                }),
            ) if level > CONFIG.controller.battery_low_threshold => {
                from_pv - to_house
                    > if matches!(self.plug.state, PlugState::On) {
                        0
                    } else {
                        CONFIG.controller.miner_demand
                    }
            }
            _ => false,
        }
    }

    pub fn should_skip_send_plug_command(&self, on: bool) -> bool {
        matches!(
            (&self.plug.state, on),
            (PlugState::Unknown, _) | (PlugState::On, true) | (PlugState::Off, false)
        )
    }
}
