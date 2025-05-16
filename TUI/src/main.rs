use messages_common::{connect_jetstream, connect_nats, create_stream, try_create_stream};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let nats = connect_nats().await;
    let jetstream = connect_jetstream().await;
    let stream = try_create_stream(&jetstream, "hello")
        .await
        .expect("could not create stream");
}
