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
use serialport::SerialPort;
use std::sync::Arc;
use tracing::{debug, error};

use super::{
    com_process,
    message::wrc::WRCPacket,
    sm7bits::{self, SM7BitControlBits, SM_7_BIT_END_BYTE},
};

fn open_port<'a>(
    port: impl Into<std::borrow::Cow<'a, str>>,
    exit_required: Arc<AtomicBool>,
) -> Option<Box<dyn SerialPort>> {
    let port = port.into();

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(p) = serialport::new(port.clone(), 115_200)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            return Some(p);
        } else {
            error!("Cannot open port {}, will retry after 1 sec", port);
            std::thread::sleep(Duration::from_millis(1000));
        }
    }

    None
}

fn read_packet(
    exit_required: Arc<AtomicBool>,
    port: &mut Box<dyn serialport::SerialPort>,
) -> Vec<u8> {
    let mut readed = vec![];
    let mut serial_buf = [0];

    while !exit_required.load(Ordering::Acquire) {
        match port.read(&mut serial_buf) {
            Ok(readed_size) => {
                if readed_size == 0 {
                    break;
                }
                if readed.is_empty()
                    && serial_buf[0] != SM7BitControlBits::USBLocal as u8
                    && serial_buf[0] != SM7BitControlBits::WRC as u8
                {
                    continue;
                }

                readed.push(serial_buf[0]);
                if serial_buf[0] == SM_7_BIT_END_BYTE {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    readed
}

fn reader(exit_required: Arc<AtomicBool>, port: &mut Box<dyn SerialPort>) -> Option<WRCPacket> {
    let readed = read_packet(exit_required, port);
    if readed.is_empty() {
        return None;
    }

    match sm7bits::decode(&readed) {
        Ok((SM7BitControlBits::WRC, decoded)) => match WRCPacket::try_from(decoded) {
            Ok(p) => Some(p),
            Err(e) => {
                error!("Cannot parse: {readed:02X?}, reason: {e}");
                None
            }
        },
        Err(e) => {
            error!("Cannot decode: {readed:02X?}, reason: {e}");
            None
        }
        _ => None,
    }
}

pub fn read_write_loop<'a>(
    rx: mpsc::Receiver<WRCPacket>,
    tx: mpsc::Sender<WRCPacket>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    exit_required: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let port = port.into();
    let mut opened_port = None;

    debug!("Starting read write loop on port {}", port);
    while !exit_required.load(Ordering::Acquire) {
        if opened_port.is_none() {
            opened_port = open_port(port.clone(), exit_required.clone());
            if opened_port.is_some() {
                debug!("Port: {} opened", port);
            }
            continue;
        }

        if let Some(readed) = reader(exit_required.clone(), opened_port.as_mut().unwrap()) {
            tx.send(readed)?;
        }

        if let Ok(packet) = rx.try_recv() {
            match TryInto::<Vec<u8>>::try_into(packet) {
                Ok(data) => {
                    let encoded = sm7bits::encode(&data, SM7BitControlBits::WRC);
                    if let Err(e) = opened_port.as_mut().unwrap().write_all(&encoded) {
                        error!("Cannot write to port: {}", e);
                        opened_port = None;
                    }
                }
                Err(e) => {
                    error!("Cannot convert packet to bytes: {}", e);
                }
            }
        }
    }

    Ok(())
}

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
        }
    }
}
