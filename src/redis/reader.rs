use std::sync::atomic::{AtomicBool, Ordering};

use tracing::debug;

use super::message::ConnectResqust;
use crate::AppConfig;
use std::sync::Arc;

pub fn read_redis(exit_required: Arc<AtomicBool>, config: &AppConfig) -> anyhow::Result<()> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    pubsub.subscribe(&config.database.queue)?;
    pubsub.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

    debug!(
        "Redis reader listening on {}/{}",
        config.database.uri, config.database.queue
    );
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(msg) = pubsub.get_message() {
            let payload: String = msg.get_payload()?;
            debug!("channel '{}': \"{}\"", msg.get_channel_name(), payload);
            let parsed: ConnectResqust = serde_json::from_str(&payload)?;
            debug!("parsed: {:?}", parsed);
        }
    }

    Ok(())
}
