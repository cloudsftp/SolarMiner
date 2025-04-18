pub trait SiteDataProvider {
    async fn get_current_excess_power(&self) -> f64;
}

pub mod solaredge {
    use super::SiteDataProvider;

    use reqwest::Client;
    use types::CurrentPowerFlow;

    const API_URL: &str = "https://monitoringapi.solaredge.com/site/";

    pub struct SolarEdgeDataProvider {
        client: Client,
        api_key: String,
        site_id: String,
    }

    impl SolarEdgeDataProvider {
        pub fn new(api_key: String, site_id: String) -> Self {
            let client = Client::new();

            Self {
                client,
                api_key,
                site_id,
            }
        }

        fn get_current_power_flow(&self) -> CurrentPowerFlow {
            todo!("send reqwest and then decode json")
        }
    }

    impl SiteDataProvider for SolarEdgeDataProvider {
        async fn get_current_excess_power(&self) -> f64 {
            todo!("first get current power flow, then return excess power")
        }
    }

    pub mod types {
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
            unit: String,
            connections: Vec<Connection>,
            #[serde(flatten)]
            devices: DevicePowerFlows,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub struct Connection {
            from: DeviceIdentifier,
            to: DeviceIdentifier,
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub enum DeviceIdentifier {
            Grid,
            Load,
            PV,
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
                let json_file_path = "responses/currentPowerFlow/charging.json";
                let mut json_file =
                    File::open(json_file_path).expect("could not open file with test data");
                let mut contents = String::new();
                json_file
                    .read_to_string(&mut contents)
                    .expect("could not read contents of file");

                let current_power_flow: CurrentPowerFlowWrapper =
                    serde_json::from_str(&contents).expect("could not convert content to struct");

                assert_eq!(
                    current_power_flow,
                    CurrentPowerFlowWrapper {
                        site_current_power_flow: CurrentPowerFlow {
                            update_refresh_rate: 3,
                            unit: String::from("kW"),
                            connections: vec![
                                Connection {
                                    from: DeviceIdentifier::PV,
                                    to: DeviceIdentifier::Storage
                                },
                                Connection {
                                    from: DeviceIdentifier::PV,
                                    to: DeviceIdentifier::Load
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
                                }
                            },
                        }
                    }
                )
            }
        }
    }
}
