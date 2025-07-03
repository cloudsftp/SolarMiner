use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateUpdateEvent {
    MinerSwitch {
        on: bool,
    },
    MinerPower {
        power: f32,
        total_energy: f32,
    },
    HouseBattery {
        level: f32,
    },
    HousePower {
        from_pv: f32,
        from_battery: f32,
        from_grid: f32,
        to_house: f32,
        to_battery: f32,
        to_grid: f32,
    },
}
