use super::{EnergyState, PartialState, PowerData};

use solarminer_service::events::StateUpdateEvent;

impl PartialState {
    pub fn get_state_update_events(&self) -> Vec<StateUpdateEvent> {
        let mut events = vec![];

        if let Some(on) = self.plug.on.get_option() {
            events.push(StateUpdateEvent::MinerSwitch { on });
        }

        if let Some(EnergyState { total, power, .. }) = self.plug.energy.get_option() {
            events.push(StateUpdateEvent::MinerPower {
                power,
                total_energy: total,
            });
        }

        if let Some(level) = self.inverter.battery_level.get_option() {
            events.push(StateUpdateEvent::HouseBattery { level });
        }

        if let Some(PowerData {
            from_pv,
            from_battery,
            from_grid,
            to_house,
            to_battery,
            to_grid,
        }) = self.inverter.power.get_option()
        {
            events.push(StateUpdateEvent::HousePower {
                from_pv,
                from_battery,
                from_grid,
                to_house,
                to_battery,
                to_grid,
            });
        }

        events
    }
}
