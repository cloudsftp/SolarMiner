use super::{EnergyState, PartialState, PowerData};

use chrono::Utc;
use solarminer_service::events::{StateUpdateEvent, StateUpdateEventMessage};

impl PartialState {
    pub fn get_state_update_events(&self) -> Vec<StateUpdateEventMessage> {
        let timestamp = Utc::now();

        let mut events = vec![];
        let mut add_event = |event: StateUpdateEvent| {
            events.push(StateUpdateEventMessage { timestamp, event });
        };

        if let Some(on) = self.plug.on.get_option() {
            add_event(StateUpdateEvent::MinerSwitch { on });
        }

        if let Some(EnergyState { total, power, .. }) = self.plug.energy.get_option() {
            add_event(StateUpdateEvent::MinerPower {
                power,
                total_energy: total,
            });
        }

        if let Some(level) = self.inverter.battery_level.get_option() {
            add_event(StateUpdateEvent::HouseBattery { level });
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
            add_event(StateUpdateEvent::HousePower {
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
