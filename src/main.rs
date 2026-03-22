mod analysis;
mod constants;
mod context;
mod doc;
mod documents;
mod handlers;
mod http_client;
mod server;

#[tokio::main]
async fn main() {
    server::run().await;
}
