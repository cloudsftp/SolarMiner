use super::SiteDataProvider;

use anyhow::{Error, anyhow};
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use types::{CurrentPowerFlow, CurrentPowerFlowWrapper};

pub mod types;

const API_URL: &str = "https://monitoringapi.solaredge.com/";

pub struct SolarEdgeDataProvider {
    client: Client,
    base_url: Url,
    api_key: String,
    site_id: String,
}

impl SolarEdgeDataProvider {
    pub fn new(api_key: String, site_id: String) -> Self {
        let client = Client::new();
        let base_url = Url::parse(API_URL).expect("parsing base api url failed");

        Self {
            client,
            base_url,
            api_key,
            site_id,
        }
    }

    fn site_url(&self, property: &str) -> Result<Url, Error> {
        self.base_url
            .join(&format!("site/{}/", self.site_id))
            .map_err(|err| {
                anyhow!(
                    "could not add site id ({}) to the base url ({}): {}",
                    self.site_id,
                    self.base_url,
                    err,
                )
            })?
            .join(property)
            .map_err(|err| {
                anyhow!(
                    "could not add identifier ({}) to the base url ({}): {}",
                    property,
                    self.base_url,
                    err,
                )
            })
    }

    async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        Ok(self
            .client
            .get(url.clone())
            .query(&[("api_key", &self.api_key)])
            .send()
            .await
            .map_err(|err| anyhow!("error sending get request to {}: {}", url, err))?
            .json()
            .await
            .map_err(|err| anyhow!("could not decode json response: {}", err))?)
    }

    async fn get_current_power_flow(&self) -> Result<CurrentPowerFlow, Error> {
        Ok(self
            .get_json::<CurrentPowerFlowWrapper>(self.site_url("currentPowerFlow")?)
            .await?
            .site_current_power_flow)
    }
}

impl SiteDataProvider for SolarEdgeDataProvider {
    async fn get_current_excess_power(&self) -> Result<f64, Error> {
        let devices = self
            .get_current_power_flow()
            .await
            .map_err(|err| anyhow!("could not get the current power flow: {}", err,))?
            .devices;
        Ok(devices.pv.current_power - devices.load.current_power)
    }
}
