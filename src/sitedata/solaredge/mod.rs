use super::SiteDataProvider;

use reqwest::Client;
use types::CurrentPowerFlow;

pub mod types;

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
