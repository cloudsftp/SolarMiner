use rumqttc::Publish;
use serde_json::{Value, json};

use super::*;

#[test]
fn decode_command_result() {
    struct TestCase<'a> {
        name: &'a str,
        payload: Value,
        expected: CommandResult,
    }

    let test_cases = [TestCase {
        name: "power turned on",
        payload: json!({"POWER": "ON"}),
        expected: CommandResult::Power(PowerUpdateValue::On),
    }];

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
fn decode_power_event() {
    struct TestCase<'a> {
        name: &'a str,
        topic: &'a str,
        payload: &'a str,
        expected: UpdateEvent,
    }

    let test_cases = [
        TestCase {
            name: "simple on",
            topic: "stat/power_device/POWER",
            payload: "ON",
            expected: UpdateEvent::PlugUpdate {
                device: "power_device".into(),
                on: true,
            },
        },
        TestCase {
            name: "simple off",
            topic: "stat/power_device/POWER",
            payload: "OFF",
            expected: UpdateEvent::PlugUpdate {
                device: "power_device".into(),
                on: false,
            },
        },
    ];

    for TestCase {
        name,
        topic,
        payload,
        expected,
    } in test_cases
    {
        let event = Publish {
            dup: false,
            qos: rumqttc::QoS::AtLeastOnce,
            retain: false,
            topic: topic.into(),
            pkid: 4444,
            payload: payload.into(),
        };

        let decoded = UpdateEvent::try_from(event)
            .expect(format!("could not decode event in test case '{}'", name).as_str());

        assert_eq!(decoded, expected, "in test case '{}'", name);
    }
}
