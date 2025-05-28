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
        TestCase {
            name: "energy usage query \"EnergyTotal\"",
            payload: json!({"EnergyTotal": {"total": 3., "yesterday": 2., "today": 1.}}),
            expected: CommandResult::EnergyConsumption {
                total: 3.,
                yesterday: 2.,
                today: 1.,
            },
        },
        TestCase {
            name: "energy usage query \"EnergyYesterday\"",
            payload: json!({"EnergyYesterday": {"total": 3., "yesterday": 2., "today": 1.}}),
            expected: CommandResult::EnergyConsumption {
                total: 3.,
                yesterday: 2.,
                today: 1.,
            },
        },
        TestCase {
            name: "energy usage query \"EnergyToday\"",
            payload: json!({"EnergyToday": {"total": 3., "yesterday": 2., "today": 1.}}),
            expected: CommandResult::EnergyConsumption {
                total: 3.,
                yesterday: 2.,
                today: 1.,
            },
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
            .expect(format!("could not decode payload in test case '{}'", name).as_str());

        assert_eq!(decoded, expected, "test case '{}'", name);
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
            .expect(format!("could not decode event in test case '{}'", name).as_str());

        assert_eq!(decoded, expected, "in test case '{}'", name);
    }
}
