use redis::Commands;
use tracing::{debug, error};
use uuid::Uuid;

use crate::message::ResponseAction;
use crate::redis::message::{BindResponse, BindResponseMsg, ConnectResponse, ConnectResponseMsg};
use crate::AppConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

fn main_loop(
    config: &AppConfig,
    mut con: redis::Connection,
    exit_required: Arc<AtomicBool>,
    rx: &mpsc::Receiver<ResponseAction>,
) -> anyhow::Result<()> {
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
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_SERIAL_INIT_RECEIVE".to_string(),
                        current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        msg_txt: BindResponseMsg {
                            product_serial_no: info.connect_id,
                            serial_no: format!("{:X}", info.wrench_serial),
                            msg_id: info.msg_id,
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
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_CONNECTION_ASK".to_string(),
                        current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        msg_txt: ConnectResponseMsg {
                            wrench_serial: format!("{:X}", info.wrench_serial),
                            status: if info.status { "0" } else { "1" }.to_string(),
                            desc: if info.status {
                                "连接成功"
                            } else {
                                "连接失败"
                            }
                            .to_string(),
                            msg_id: info.msg_id,
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

fn get_con(config: &AppConfig) -> anyhow::Result<redis::Connection> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let con = client.get_connection()?;

    Ok(con)
}

pub fn write_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    rx: mpsc::Receiver<ResponseAction>,
) {
    while !exit_required.load(Ordering::Acquire) {
        match get_con(config) {
            Ok(con) => {
                if let Err(e) = main_loop(config, con, exit_required.clone(), &rx) {
                    error!("redis writer error: {}", e);
                }
            }
            Err(e) => {
                error!("redis connection error: {}", e);
                std::thread::yield_now();
            }
        }
    }
}
