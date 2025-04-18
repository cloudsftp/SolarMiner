pub mod solaredge;

pub trait SiteDataProvider {
    async fn get_current_excess_power(&self) -> f64;
}
