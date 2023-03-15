use redis::Commands;
use tracing::debug;

use crate::message::Action;
use crate::redis::message::{ConnectResponse, ConnectResponseMsg};
use crate::AppConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

pub fn write_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    rx: mpsc::Receiver<Action>,
) -> anyhow::Result<()> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let mut con = client.get_connection()?;
    debug!(
        "Redis writer listening on {}/{}",
        config.database.uri, config.database.queue
    );
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(msg) = rx.try_recv() {
            if let Action::ConnectStatus((status, info)) = msg {
                let connect_response = ConnectResponse {
                    msg_type: "0".to_string(),
                    msg_id: info.msg_id,
                    handler_name: "TOPIC_WRENCH_CONNECTION_ASK".to_string(),
                    msg_txt: ConnectResponseMsg {
                        wrench_name: None,
                        wrench_serial: Some(format!("{:X}", info.wrench_serial)),
                        status: Some(if status { "0" } else { "1" }.to_string()),
                        desc: None,
                        current_time: None,
                        task_id: None,
                    },
                };
                con.publish(
                    config.database.queue.as_str(),
                    serde_json::to_string(&connect_response)?,
                )?;
            }
        }
    }

    Ok(())
}
