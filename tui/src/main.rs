use env_logger::Target;
use nats_common::connect_nats;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().target(Target::Stdout).init();

    let nats = connect_nats().await;
}
