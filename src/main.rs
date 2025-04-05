use std::sync::Arc;

pub mod app;
pub mod ports;
pub mod di;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let container = Arc::new(di::Container::new(app::query::get_hello_world::InMemoryRepository));
    let server = ports::http_api::Server::new(3000, container);
    server.run().await;
}
