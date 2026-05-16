use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr: SocketAddr = std::env::var("MTGDECKBUILDER_WEB_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8765".to_string())
        .parse()?;
    mtgdeckbuilder::web::run(addr).await
}
