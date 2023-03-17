use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use serde_json::Value;
use tracing::{debug, error};

use super::message::ConnectRequest;
use crate::{
    message::{ConnectInfo, RequiredAction, WrenchInfo},
    redis::message::{BindRequest, TaskRequest},
    AppConfig,
};
use std::sync::Arc;

pub fn read_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    tx: mpsc::Sender<RequiredAction>,
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
            let parsed: Value = match serde_json::from_str(&payload) {
                Ok(v) => v,
                Err(e) => {
                    error!("invalid json format: {}", e);
                    continue;
                }
            };

            match parsed.get("handlerName") {
                Some(Value::String(s)) if s == "TOPIC_WRENCH_SERIAL_INIT" => {
                    let bind_request: BindRequest = match serde_json::from_str(&payload) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("invalid json format: {}", e);
                            continue;
                        }
                    };
                    debug!("bind request: {:?}", bind_request);
                    tx.send(RequiredAction::BindWrench(WrenchInfo {
                        msg_id: bind_request.msg_id,
                        connect_id: bind_request.msg_txt.product_serial_no,
                        ..Default::default()
                    }))?;
                }
                Some(Value::String(s)) if s == "TOPIC_WRENCH_CONNECTION" => {
                    let connect_request: ConnectRequest = match serde_json::from_str(&payload) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("invalid json format: {}", e);
                            continue;
                        }
                    };
                    debug!("connect request: {:?}", connect_request);
                    match u128::from_str_radix(&connect_request.msg_txt.wrench_serial, 16) {
                        Ok(s) => {
                            tx.send(RequiredAction::CheckConnect(ConnectInfo {
                                msg_id: connect_request.msg_id,
                                wrench_serial: s,
                                ..Default::default()
                            }))?;
                        }
                        Err(_) => error!("invalid serial number"),
                    }
                }
                Some(Value::String(s)) if s == "TOPIC_WRENCH_TASK_UP_SEND" => {
                    let task_request: TaskRequest = match serde_json::from_str(&payload) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("invalid json format: {}", e);
                            continue;
                        }
                    };
                    debug!("task request: {:?}", task_request);
                }
                _ => {
                    error!("unknown message type");
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    Ok(())
}
