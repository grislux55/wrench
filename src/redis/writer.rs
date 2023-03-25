use redis::Commands;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::message::ResponseAction;
use crate::redis::message::{
    BindResponse, BindResponseMsg, ConnectResponse, ConnectResponseMsg, TaskResponse,
    TaskResponseMsg,
};
use crate::AppConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

fn publish_msg<T: serde::Serialize>(
    con: &mut redis::Connection,
    queue: &str,
    msg: T,
) -> anyhow::Result<()> {
    let msg = serde_json::to_string(&msg)?;
    info!("发布消息: {} 到 Redis", msg);
    con.publish(queue, msg)?;
    Ok(())
}

fn main_loop(
    config: &AppConfig,
    mut con: redis::Connection,
    exit_required: Arc<AtomicBool>,
    rx: &mpsc::Receiver<ResponseAction>,
) -> anyhow::Result<()> {
    info!(
        "已在目标 Redis: {} 上的 {} 队列进行发布循环, 将在收取主线程的数据之后进行发布",
        config.database.uri, config.database.queue
    );
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(msg) = rx.try_recv() {
            if cfg!(debug_assertions) {
                debug!("收到主线程的消息: {:?}", msg);
            } else {
                info!("收到主线程的消息: {}", msg);
            }
            match msg {
                ResponseAction::BindResponse(info) => {
                    let bind_response = BindResponse {
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_SERIAL_INIT_ASK".to_string(),
                        current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        msg_txt: BindResponseMsg {
                            product_serial_no: info.connect_id,
                            wrench_serial: format!("{:X}", info.wrench_serial),
                            msg_id: info.msg_id,
                        },
                    };
                    publish_msg(&mut con, config.database.queue.as_str(), bind_response)?;
                }
                ResponseAction::ConnectStatus(info) => {
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
                    publish_msg(&mut con, config.database.queue.as_str(), connect_response)?;
                }
                ResponseAction::TaskStatus(info) => {
                    let task_response = TaskResponse {
                        msg_id: Uuid::new_v4().simple().to_string(),
                        handler_name: "TOPIC_WRENCH_TASK_UP_ASK".to_string(),
                        current_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        msg_txt: TaskResponseMsg {
                            wrench_serial: format!("{:X}", info.wrench_serial),
                            status: if info.status { "0" } else { "1" }.to_string(),
                            desc: if info.status {
                                "接受成功"
                            } else {
                                "接受失败"
                            }
                            .to_string(),
                            msg_id: info.msg_id,
                        },
                    };
                    publish_msg(&mut con, config.database.queue.as_str(), task_response)?;
                }
            }
        } else {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    Ok(())
}

fn get_con(config: &AppConfig) -> anyhow::Result<redis::Connection> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let con = client.get_connection()?;

    info!("已连接到 Redis: {}", config.database.uri);
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
                    error!("Redis 发布线程出现错误: {}, 尝试重新获取 Redis 连接", e);
                }
            }
            Err(e) => {
                error!("无法连接到 Redis, 原因: {}, 将在 1 秒后进行重连", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
