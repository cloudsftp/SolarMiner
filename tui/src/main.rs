use nats_common::connect_nats;

#[tokio::main]
async fn main() {
    env_logger::init();

    let nats = connect_nats().await;
}
