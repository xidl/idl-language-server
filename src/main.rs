mod analysis;
mod constants;
mod context;
mod documents;
mod handlers;
mod doc;
mod http_client;
mod server;

#[tokio::main]
async fn main() {
    server::run().await;
}
