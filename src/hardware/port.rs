use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use crate::message::{RequiredAction, ResponseAction};

use bus::Bus;
use std::sync::Arc;
use tracing::{debug, error, info, span, Level};

use super::com_process;

fn create_com_thread(
    exit_required: Arc<AtomicBool>,
    port: String,
    tx: mpsc::Sender<ResponseAction>,
    bus: Arc<Mutex<Bus<RequiredAction>>>,
) -> anyhow::Result<JoinHandle<()>> {
    let mut bus = bus.lock().map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let rx = bus.add_rx();
    drop(bus);

    let handle = std::thread::spawn(move || {
        span!(Level::ERROR, "串口处理线程", port = %port).in_scope(|| {
            com_process::com_process(exit_required, &port, tx, rx);
        });
    });

    Ok(handle)
}

pub fn loop_query(
    exit_required: Arc<AtomicBool>,
    tx: mpsc::Sender<ResponseAction>,
    bus: Arc<Mutex<Bus<RequiredAction>>>,
) {
    let mut com_thread_handles: Vec<(String, JoinHandle<()>)> = vec![];

    info!("开始进行串口监听");
    while !exit_required.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_secs(1));

        com_thread_handles.retain(|(_, h)| !h.is_finished());

        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                error!("无法枚举所有串口: {}", e);
                continue;
            }
        };
        debug!(
            "当前所有串口: {:?}",
            ports
                .iter()
                .map(|p| p.port_name.clone())
                .collect::<Vec<_>>()
        );

        for p in ports.iter() {
            if com_thread_handles
                .iter()
                .any(|(port, _)| port == &p.port_name)
            {
                continue;
            }
            info!("新的串口: {}, 创建处理线程", p.port_name);
            match create_com_thread(
                exit_required.clone(),
                p.port_name.clone(),
                tx.clone(),
                bus.clone(),
            ) {
                Ok(h) => com_thread_handles.push((p.port_name.clone(), h)),
                Err(e) => error!("无法创建串口处理线程: {}", e),
            }
        }
    }

    for (_, h) in com_thread_handles {
        h.join().ok();
    }
}
