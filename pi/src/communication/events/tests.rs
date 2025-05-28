use std::fs::File;

/*
#[test]
fn battery_state() {
    let file =
        File::open("data/solaredge/battery_discharging_99.json").expect("could not open file");

    let battery_state: BatteryState =
        serde_json::from_reader(file).expect("could not decode battery state");

    dbg!(&battery_state);

    assert_eq!(
        BatteryState {
            status: BatteryStatus::Discharging,
            state_of_charge: 98.89,
        },
        battery_state
    )
}
 */
