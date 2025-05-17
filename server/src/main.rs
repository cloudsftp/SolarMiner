use anyhow::Error;
use dotenv::dotenv;
use sitedata::{SiteDataProvider, solaredge::SolarEdgeDataProvider};
use std::env;

mod sitedata;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv()?;

    let api_key = env::var("API_KEY")?;
    let site_id = env::var("SITE_ID")?;

    let data_provider = SolarEdgeDataProvider::new(api_key, site_id);

    let excess_power = data_provider.get_current_excess_power().await?;

    dbg!(excess_power);

    /*
        let base_url = format!("{}{}/", API_URL, site_id);
        let url = format!("{}currentPowerFlow?api_key={}", base_url, api_key);
    */

    Ok(())
}
