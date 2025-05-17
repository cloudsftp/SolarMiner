use anyhow::Error;

pub mod solaredge;

pub trait SiteDataProvider {
    async fn get_current_excess_power(&self) -> Result<f64, Error>;
}
