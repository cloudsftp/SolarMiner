use std::fs::File;

use super::*;

#[test]
fn battery_state() {
    struct TestCase<'a> {
        name: &'a str,
        payload_file_name: &'a str,
        expected: BatteryState,
    }

    let test_cases = [
        TestCase {
            name: "discharging",
            payload_file_name: "data/solaredge/battery_discharging_99.json",
            expected: BatteryState {
                status: BatteryStatus::Discharging,
                state_of_charge: 98.89,
            },
        },
        TestCase {
            name: "preserving charge",
            payload_file_name: "data/solaredge/battery_preserving_charge.json",
            expected: BatteryState {
                status: BatteryStatus::PreservingCharge,
                state_of_charge: 100.00,
            },
        },
    ];

    for TestCase {
        name,
        payload_file_name,
        expected,
    } in test_cases
    {
        let file = File::open(payload_file_name)
            .unwrap_or_else(|_| panic!("could not open file '{payload_file_name}'"));

        let battery_state: BatteryState = serde_json::from_reader(file).unwrap_or_else(|_| panic!("could not decode battery state from file '{payload_file_name}'"));

        assert_eq!(battery_state, expected, "in test case '{name}'")
    }
}
