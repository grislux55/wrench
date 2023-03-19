use std::{
    collections::HashMap,
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
use tracing::{debug, error};

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
        if let Err(e) = com_process::com_process(exit_required, port, tx, rx) {
            error!("Com thread error: {}", e);
        }
    });

    Ok(handle)
}

pub fn loop_query(
    exit_required: Arc<AtomicBool>,
    tx: mpsc::Sender<ResponseAction>,
    bus: Arc<Mutex<Bus<RequiredAction>>>,
) {
    let mut last_com_threads = HashMap::new();
    let mut com_thread_handles = vec![];

    while !exit_required.load(Ordering::Acquire) {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                error!("Can't enum all available ports: {}", e);
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        for p in ports.iter() {
            if !last_com_threads.contains_key(&p.port_name) {
                debug!("New port: {}", p.port_name);
                match create_com_thread(
                    exit_required.clone(),
                    p.port_name.clone(),
                    tx.clone(),
                    bus.clone(),
                ) {
                    Ok(h) => com_thread_handles.push((p.port_name.clone(), h)),
                    Err(e) => error!("Can't create com thread: {}", e),
                }
            }
        }

        last_com_threads.clear();
        last_com_threads.extend(ports.into_iter().map(|p| (p.port_name, ())));

        std::thread::sleep(Duration::from_secs(1));
    }

    for (s, h) in com_thread_handles {
        if let Err(e) = h.join() {
            error!("Can't join com thread: {} {:?}", s, e);
        };
    }
}
