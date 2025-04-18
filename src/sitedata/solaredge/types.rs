use serde::{Deserialize, Serialize};

// TODO: get rid of ugly wrapper
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentPowerFlowWrapper {
    pub site_current_power_flow: CurrentPowerFlow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentPowerFlow {
    update_refresh_rate: u64,
    unit: PowerUnit,
    connections: Vec<Connection>,
    #[serde(flatten)]
    devices: DevicePowerFlows,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PowerUnit {
    W,
    #[serde(alias = "kW")]
    KW,
    MW,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    from: DeviceIdentifier,
    to: DeviceIdentifier,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DeviceIdentifier {
    PV,
    #[serde(alias = "GRID")]
    Grid,
    #[serde(alias = "LOAD")]
    Load,
    #[serde(alias = "STORAGE")]
    Storage,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct DevicePowerFlows {
    grid: DevicePowerFlow,
    load: DevicePowerFlow,
    pv: DevicePowerFlow,
    storage: StoragePowerFlow,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevicePowerFlow {
    status: DevicePowerFlowStatus,
    current_power: f64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DevicePowerFlowStatus {
    Active,
    Idle,
    Disabled,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePowerFlow {
    status: StoragePowerFlowStatus,
    current_power: f64,
    charge_level: u64,
    critical: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum StoragePowerFlowStatus {
    Charging,
    Discharging,
    Idle,
    Disabled,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn deser_current_power_flow() {
        struct TestCase<'a> {
            json_file_path: &'a str,
            expected: CurrentPowerFlowWrapper,
        }

        let test_cases = [
            TestCase {
                json_file_path: "responses/currentPowerFlow/charging.json",
                expected: CurrentPowerFlowWrapper {
                    site_current_power_flow: CurrentPowerFlow {
                        update_refresh_rate: 3,
                        unit: PowerUnit::KW,
                        connections: vec![
                            Connection {
                                from: DeviceIdentifier::PV,
                                to: DeviceIdentifier::Storage,
                            },
                            Connection {
                                from: DeviceIdentifier::PV,
                                to: DeviceIdentifier::Load,
                            },
                        ],
                        devices: DevicePowerFlows {
                            grid: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 0.,
                            },
                            load: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 0.19,
                            },
                            pv: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 0.73,
                            },
                            storage: StoragePowerFlow {
                                status: StoragePowerFlowStatus::Charging,
                                current_power: 0.54,
                                charge_level: 8,
                                critical: false,
                            },
                        },
                    },
                },
            },
            TestCase {
                json_file_path: "responses/currentPowerFlow/idle.json",
                expected: CurrentPowerFlowWrapper {
                    site_current_power_flow: CurrentPowerFlow {
                        update_refresh_rate: 3,
                        unit: PowerUnit::KW,
                        connections: vec![
                            Connection {
                                from: DeviceIdentifier::PV,
                                to: DeviceIdentifier::Load,
                            },
                            Connection {
                                from: DeviceIdentifier::Grid,
                                to: DeviceIdentifier::Load,
                            },
                        ],
                        devices: DevicePowerFlows {
                            grid: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 3.26,
                            },
                            load: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 4.82,
                            },
                            pv: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 1.56,
                            },
                            storage: StoragePowerFlow {
                                status: StoragePowerFlowStatus::Idle,
                                current_power: 0.,
                                charge_level: 0,
                                critical: false,
                            },
                        },
                    },
                },
            },
            TestCase {
                json_file_path: "responses/currentPowerFlow/discharging.json",
                expected: CurrentPowerFlowWrapper {
                    site_current_power_flow: CurrentPowerFlow {
                        update_refresh_rate: 3,
                        unit: PowerUnit::KW,
                        connections: vec![
                            Connection {
                                from: DeviceIdentifier::PV,
                                to: DeviceIdentifier::Load,
                            },
                            Connection {
                                from: DeviceIdentifier::Storage,
                                to: DeviceIdentifier::Load,
                            },
                        ],
                        devices: DevicePowerFlows {
                            grid: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 0.0,
                            },
                            load: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 2.38,
                            },
                            pv: DevicePowerFlow {
                                status: DevicePowerFlowStatus::Active,
                                current_power: 0.32,
                            },
                            storage: StoragePowerFlow {
                                status: StoragePowerFlowStatus::Discharging,
                                current_power: 2.06,
                                charge_level: 2,
                                critical: false,
                            },
                        },
                    },
                },
            },
        ];

        for TestCase {
            json_file_path,
            expected,
        } in test_cases
        {
            let mut json_file =
                File::open(json_file_path).expect("could not open file with test data");
            let mut contents = String::new();
            json_file
                .read_to_string(&mut contents)
                .expect("could not read contents of file");

            let current_power_flow: CurrentPowerFlowWrapper =
                serde_json::from_str(&contents).expect("could not convert content to struct");

            assert_eq!(current_power_flow, expected);
        }
    }
}
