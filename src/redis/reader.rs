use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use serde_json::Value;
use tracing::{debug, error};

use super::message::ConnectResqust;
use crate::{
    message::{Action, ConnectInfo},
    redis::message::TaskRequest,
    AppConfig,
};
use std::sync::Arc;

pub fn read_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    tx: mpsc::Sender<Action>,
) -> anyhow::Result<()> {
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
            let parsed: Value = serde_json::from_str(&payload)?;
            match parsed.get("handlerName") {
                Some(Value::String(s)) if s == "TOPIC_WRENCH_CONNECTION" => {
                    let connect_request: ConnectResqust = serde_json::from_str(&payload)?;
                    debug!("connect request: {:?}", connect_request);
                    if let Some(serial) = connect_request.msg_txt.wrench_serial {
                        match u128::from_str_radix(&serial, 16) {
                            Ok(s) => {
                                tx.send(Action::CheckConnect(ConnectInfo {
                                    msg_id: connect_request.msg_id,
                                    wrench_serial: s,
                                }))?;
                            }
                            Err(_) => error!("invalid serial number"),
                        }
                    }
                }
                Some(Value::String(s)) if s == "TOPIC_WRENCH_TASK_UP_SEND" => {
                    let task_request: TaskRequest = serde_json::from_str(&payload)?;
                    debug!("task request: {:?}", task_request);
                }
                _ => {}
            }
        } else {
            std::thread::yield_now();
        }
    }

    Ok(())
}
