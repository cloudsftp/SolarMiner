use std::fs::File;

use serde_json::{Value, json};

use super::*;

#[test]
fn command_results_decoding() {
    struct TestCase<'a> {
        name: &'a str,
        payload: Value,
        expected: CommandResult,
    }

    let test_cases = [
        TestCase {
            name: "power turned on",
            payload: json!({"POWER": "ON"}),
            expected: CommandResult::Power(PlugStateValue::On),
        },
        TestCase {
            name: "power turned off",
            payload: json!({"POWER": "OFF"}),
            expected: CommandResult::Power(PlugStateValue::Off),
        },
    ];

    for TestCase {
        name,
        payload,
        expected,
    } in test_cases
    {
        let payload = payload.to_string();
        let decoded: CommandResult = serde_json::from_str(&payload)
            .unwrap_or_else(|_| panic!("could not decode payload in test case '{name}'"));

        assert_eq!(decoded, expected, "test case '{name}'");
    }
}

#[test]
fn update_events() {
    struct TestCase<'a> {
        name: &'a str,
        subject: &'a str,
        payload: &'a str,
        expected: UpdateEvent,
    }

    let test_cases = [
        TestCase {
            name: "simple on",
            subject: "stat.power_device.POWER",
            payload: "ON",
            expected: UpdateEvent::PlugStateUpdate {
                device: "power_device".into(),
                on: true,
            },
        },
        TestCase {
            name: "simple off",
            subject: "stat.power_device.POWER",
            payload: "OFF",
            expected: UpdateEvent::PlugStateUpdate {
                device: "power_device".into(),
                on: false,
            },
        },
    ];

    for TestCase {
        name,
        subject,
        payload,
        expected,
    } in test_cases
    {
        let message = Message {
            subject: subject.into(),
            payload: payload.into(),
            length: payload.len(),
            reply: None,
            headers: None,
            status: None,
            description: None,
        };

        let decoded = UpdateEvent::try_from(&message)
            .unwrap_or_else(|_| panic!("could not decode event in test case '{name}'"));

        assert_eq!(decoded, expected, "in test case '{name}'");
    }
}

#[test]
fn status8() {
    struct TestCase<'a> {
        name: &'a str,
        payload_file_name: &'a str,
        expected: Status8,
    }

    let test_cases = [TestCase {
        name: "discharging",
        payload_file_name: "data/plug/status8_bitaxe_on.json",
        expected: Status8 {
            status_sns: StatusSNS {
                energy: Status8Energy {
                    total: 0.1732,
                    yesterday: 0.0,
                    today: 0.1732,
                    power: 26.4,
                },
            },
        },
    }];

    for TestCase {
        name,
        payload_file_name,
        expected,
    } in test_cases
    {
        let file = File::open(payload_file_name)
            .unwrap_or_else(|_| panic!("could not open file '{payload_file_name}'"));

        let status8: Status8 = serde_json::from_reader(file).unwrap_or_else(|_| panic!("could not decode status8 from file '{payload_file_name}'"));

        assert_eq!(status8, expected, "in test case '{name}'")
    }
}
