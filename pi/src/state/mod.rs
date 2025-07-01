mod part;
mod update;

#[cfg(test)]
mod tests;

use part::Part;

use crate::CONFIG;

#[derive(Debug, PartialEq, Clone)]
pub struct PartialState {
    plug: PartialPlugState,
    inverter: PartialInverterState,
}

#[derive(Debug, PartialEq, Clone)]
struct PartialPlugState {
    on: Part<bool>,
    energy: Part<EnergyState>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct EnergyState {
    total: f32,
    yesterday: f32,
    today: f32,
    power: f32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PartialInverterState {
    power: Part<PowerData>,
    battery_level: Part<f32>,
}

#[derive(Debug, PartialEq, Clone)]
struct PowerData {
    from_pv: f32,
    from_battery: f32,
    from_grid: f32,
    to_house: f32,
    to_battery: f32,
    to_grid: f32,
}

impl PartialState {
    pub fn new() -> Self {
        let timeout = CONFIG.controller.sensor_data_outdated_interval;

        Self {
            plug: PartialPlugState {
                on: Part::new(true, timeout),
                energy: Part::new(
                    EnergyState {
                        total: 0.,
                        yesterday: 0.,
                        today: 0.,
                        power: CONFIG.controller.miner_demand,
                    },
                    timeout,
                ),
            },
            inverter: PartialInverterState {
                power: Part::new(
                    PowerData {
                        from_pv: 0.,
                        from_battery: 0.,
                        from_grid: 0.,
                        to_house: 0.,
                        to_battery: 0.,
                        to_grid: 0.,
                    },
                    timeout,
                ),
                battery_level: Part::new(0., timeout),
            },
        }
    }
}

impl PartialState {
    pub fn mining_condition(&self) -> bool {
        match self.inverter.battery_level.get_or_default() {
            level if level > CONFIG.controller.battery_high_threshold => true,
            level if level > CONFIG.controller.battery_low_threshold => {
                self.production_satisfies_miner()
            }
            _ => false,
        }
    }

    fn production_satisfies_miner(&self) -> bool {
        let PowerData {
            from_pv, to_house, ..
        } = self.inverter.power.get_or_default();

        from_pv - to_house > self.miner_demand()
    }

    fn miner_demand(&self) -> f32 {
        if self.plug.on.get_or_default() {
            0.
        } else {
            CONFIG.controller.miner_demand as f32
        }
    }

    pub fn should_skip_send_plug_command(&self, desired: bool) -> bool {
        self.plug
            .on
            .get_option()
            .map(|current| current == desired)
            .unwrap_or(false)
    }
}
