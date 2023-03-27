use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use serde_json::Value;
use tracing::{debug, error, info};

use super::message::ConnectRequest;
use crate::{
    message::{ConnectInfo, RequiredAction, WrenchInfo},
    redis::message::{BindRequest, TaskCancel, TaskRequest},
    AppConfig,
};
use std::sync::Arc;

fn send_action(tx: &mpsc::Sender<RequiredAction>, action: RequiredAction) -> anyhow::Result<()> {
    info!("发送消息: {} 到主线程", action);
    tx.send(action)?;
    Ok(())
}

fn main_loop(
    config: &AppConfig,
    exit_required: Arc<AtomicBool>,
    mut con: redis::Connection,
    tx: &mpsc::Sender<RequiredAction>,
) -> anyhow::Result<()> {
    let mut pubsub = con.as_pubsub();
    pubsub.subscribe(&config.database.queue)?;
    pubsub.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

    info!(
        "已在目标 Redis: {} 上的 {} 队列进行订阅",
        config.database.uri, config.database.queue
    );
    while !exit_required.load(Ordering::Acquire) {
        let msg = match pubsub.get_message() {
            Ok(m) => m,
            Err(e) if e.is_timeout() => {
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        let payload: String = match msg.get_payload() {
            Ok(p) => p,
            Err(e) => {
                error!("错误的 Redis payload, 原因: {}", e);
                continue;
            }
        };

        debug!(
            "从 Redis 通道 {} 接受到内容: {}",
            msg.get_channel_name(),
            payload
        );
        let parsed: Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(e) => {
                error!("错误的 Json 格式, 原因: {}", e);
                continue;
            }
        };

        match parsed.get("handlerName").map(|v| {
            info!("接受到来自Redis的 {} 消息", v);
            v
        }) {
            Some(Value::String(s)) if s == "TOPIC_WRENCH_SERIAL_INIT" => {
                let bind_request: BindRequest = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("错误的 Json 格式, 原因: {}", e);
                        continue;
                    }
                };
                send_action(
                    tx,
                    RequiredAction::BindWrench(WrenchInfo {
                        msg_id: bind_request.msg_id,
                        connect_id: bind_request.msg_txt.product_serial_no,
                        ..Default::default()
                    }),
                )?;
            }
            Some(Value::String(s)) if s == "TOPIC_WRENCH_CONNECTION" => {
                let connect_request: ConnectRequest = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("错误的 Json 格式, 原因: {}", e);
                        continue;
                    }
                };
                match u128::from_str_radix(&connect_request.msg_txt.wrench_serial, 16) {
                    Ok(s) => {
                        send_action(
                            tx,
                            RequiredAction::CheckConnect(ConnectInfo {
                                msg_id: connect_request.msg_id,
                                wrench_serial: s,
                                ..Default::default()
                            }),
                        )?;
                    }
                    Err(_) => error!("序列码格式错误, 注意序列码必须为一个 128bit 的十六进制数"),
                }
            }
            Some(Value::String(s)) if s == "TOPIC_WRENCH_TASK_UP_SEND" => {
                let task_request: TaskRequest = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("错误的 Json 格式, 原因: {}", e);
                        continue;
                    }
                };
                send_action(
                    tx,
                    RequiredAction::SendTask((task_request.msg_id, task_request.msg_txt)),
                )?;
            }
            Some(Value::String(s)) if s == "TOPIC_WRENCH_TASK_CANCEL" => {
                let task_cancel: TaskCancel = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("错误的 Json 格式, 原因: {}", e);
                        continue;
                    }
                };
                send_action(
                    tx,
                    RequiredAction::TaskCancel((
                        task_cancel.msg_txt.wrench_serial,
                        task_cancel.msg_txt.task_id,
                    )),
                )?;
            }
            Some(Value::String(s))
                if s == "TOPIC_WRENCH_SERIAL_INIT_ASK"
                    || s == "TOPIC_WRENCH_CONNECTION_ASK"
                    || s == "TOPIC_WRENCH_TASK_UP_ASK"
                    || s == "TOPIC_WRENCH_WORK_COLLECTION_RECEIVE"
                    || s == "TOPIC_WRENCH_OTHER_COLLECTION_RECEIVE" => {}
            _ => {
                error!("未知的消息格式");
            }
        }
    }

    Ok(())
}

fn get_pubsub(config: &AppConfig) -> anyhow::Result<redis::Connection> {
    let client = redis::Client::open(config.database.uri.clone())?;
    let con = client.get_connection()?;

    info!("已连接到 Redis: {}", config.database.uri);
    Ok(con)
}

pub fn read_redis(
    exit_required: Arc<AtomicBool>,
    config: &AppConfig,
    tx: mpsc::Sender<RequiredAction>,
) {
    while !exit_required.load(Ordering::Acquire) {
        match get_pubsub(config) {
            Ok(con) => {
                if let Err(e) = main_loop(config, exit_required.clone(), con, &tx) {
                    error!("Redis 订阅线程出现错误: {}, 尝试重新获取 Redis 连接", e);
                }
            }
            Err(e) => {
                error!("无法连接到 Redis, 原因: {}, 将在 1 秒后进行重连", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
