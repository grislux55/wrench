use redis::Commands;
use tracing::debug;
use uuid::Uuid;

use crate::message::ResponseAction;
use crate::redis::message::{BindResponse, BindResponseMsg, ConnectResponse, ConnectResponseMsg};
use crate::AppConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

pub fn write_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    rx: mpsc::Receiver<ResponseAction>,
) -> anyhow::Result<()> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let mut con = client.get_connection()?;

    debug!(
        "Redis writer listening on {}/{}",
        config.database.uri, config.database.queue
    );
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(msg) = rx.try_recv() {
            match msg {
                ResponseAction::BindStatus(info) => {
                    debug!(
                        "bind serial: {:X} to {}",
                        info.wrench_serial, info.connect_id
                    );
                    let bind_response = BindResponse {
                        msg_type: "1".to_string(),
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_SERIAL_INIT_RECEIVE".to_string(),
                        msg_txt: BindResponseMsg {
                            product_serial_no: Some(info.connect_id),
                            serial_no: if info.wrench_serial != 0 {
                                Some(format!("{:X}", info.wrench_serial))
                            } else {
                                None
                            },
                            current_time: Some(
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            ),
                        },
                    };
                    con.publish(
                        config.database.queue.as_str(),
                        serde_json::to_string(&bind_response)?,
                    )?;
                }
                ResponseAction::ConnectStatus(info) => {
                    debug!(
                        "serial: {:X} connect status: {}",
                        info.wrench_serial, info.status
                    );
                    let connect_response = ConnectResponse {
                        msg_type: "0".to_string(),
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_CONNECTION_ASK".to_string(),
                        msg_txt: ConnectResponseMsg {
                            wrench_serial: Some(format!("{:X}", info.wrench_serial)),
                            status: Some(if info.status { "0" } else { "1" }.to_string()),
                            current_time: Some(
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            ),
                            task_id: Some(info.task_id),
                            ..Default::default()
                        },
                    };
                    con.publish(
                        config.database.queue.as_str(),
                        serde_json::to_string(&connect_response)?,
                    )?;
                }
            }
        } else {
            std::thread::yield_now();
        }
    }

    Ok(())
}
