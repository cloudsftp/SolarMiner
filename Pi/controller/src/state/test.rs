use rumqttc::Publish;
use serde_json::json;

use super::*;

#[test]
fn decode_power_event() {
    struct TestCase<'a> {
        name: &'a str,
        topic: &'a str,
        payload: String,
        expected: UpdateEvent,
    }

    fn power_payload(state: &str) -> String {
        json!({ "POWER": state }).to_string()
    }

    let test_cases = [TestCase {
        name: "simple on",
        topic: "stat/power_device/POWER",
        payload: power_payload("ON"),
        expected: UpdateEvent::PlugUpdate {
            device: "power_device".into(),
            on: true,
        },
    }];

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
