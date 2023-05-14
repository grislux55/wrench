mod message;
mod port;
mod redis;
mod wrench;

use std::{
    collections::HashMap,
    matches,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

use bus::BusReader;

use tracing::{debug, error, info, span, Level};

use crate::message::{RequiredAction, ResponseAction, WrenchInfo};

use self::{
    message::process_com_message,
    port::read_write_loop,
    redis::process_message_from_redis,
    wrench::{WrenchContext, WrenchStatus},
};

use super::message::wrc::WRCPacket;

pub struct ComProcess {
    pub reader: Receiver<WRCPacket>,
    pub writer: Sender<WRCPacket>,
    pub handle: JoinHandle<()>,
    pub wrenches_mac_map: HashMap<u32, usize>,
    pub wrenches_serial_map: HashMap<u128, usize>,
    pub wrenches: Vec<WrenchContext>,
    pub connection_pending: Vec<WrenchInfo>,
}

fn com_update(com: &mut ComProcess, tx: &mpsc::Sender<ResponseAction>) -> anyhow::Result<()> {
    for wrench in com.wrenches.iter_mut() {
        wrench.interval_update(&com.writer, tx);
        if wrench.connect_id.is_empty() && !matches!(wrench.status, WrenchStatus::Disconnected) {
            if let Some(mut wrench_info) = com.connection_pending.pop() {
                wrench.connect_id = wrench_info.connect_id.clone();
                wrench_info.wrench_serial = wrench.serial;
                tx.send(ResponseAction::BindResponse(wrench_info))?;
            }
        }
    }

    Ok(())
}

pub fn com_process<'a>(
    exit_required: Arc<AtomicBool>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    tx: mpsc::Sender<ResponseAction>,
    mut rx: BusReader<RequiredAction>,
) {
    let port = port.into();
    let mut com = {
        let (thread_writer, reader) = mpsc::channel();
        let (writer, thread_reader) = mpsc::channel();

        let handle = {
            let port = port.to_string();
            let exit_required = exit_required.clone();
            info!("启动串口读写线程");
            std::thread::spawn(move || {
                span!(Level::ERROR, "串口读写线程", port = %port).in_scope(|| {
                    read_write_loop(thread_reader, thread_writer, &port, exit_required);
                });
            })
        };
        ComProcess {
            reader,
            writer,
            handle,
            connection_pending: Vec::new(),
            wrenches_mac_map: HashMap::new(),
            wrenches_serial_map: HashMap::new(),
            wrenches: Vec::new(),
        }
    };

    info!("启动处理循环");
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(wrc) = com.reader.try_recv() {
            // debug!("收到串口消息: {:02X?}", wrc);
            if let Err(e) = process_com_message(&mut com, &wrc, &tx) {
                error!("处理串口消息失败: {}", e);
            }
        }

        if let Ok(action) = rx.try_recv() {
            debug!("收到 Redis 消息: {:02X?}", action);
            if let Err(e) = process_message_from_redis(&mut com, action, &tx) {
                error!("处理 Redis 消息失败: {}", e);
            }
        }

        if let Err(e) = com_update(&mut com, &tx) {
            error!("定时更新失败: {}", e);
        }
    }
}
