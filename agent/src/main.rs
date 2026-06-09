use std::time::Duration;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let poll_secs: u64 = std::env::var("AGENT_POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    tracing::info!("StarSight agent starting (poll interval: {}s)", poll_secs);

    loop {
        tokio::time::sleep(Duration::from_secs(poll_secs)).await;
        tracing::info!("polling for new prediction opportunities");
    }
}
