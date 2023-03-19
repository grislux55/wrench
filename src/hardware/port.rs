use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::{
    hardware::{message::wrc::WRCPacket, sm7bits::SM7BitControlBits},
    message::{RequiredAction, ResponseAction, WrenchInfo},
};

use super::message::{
    usb::USBLocalPacket,
    wrc::{WRCPacketFlag, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag},
};
use super::sm7bits;
use super::sm7bits::SM_7_BIT_END_BYTE;
use bus::{Bus, BusReader};
use either::Either;
use serialport::SerialPort;
use std::sync::Arc;
use tracing::{debug, error};

fn read_packet(port: &mut Box<dyn serialport::SerialPort>) -> Vec<Vec<u8>> {
    let mut readed = vec![];
    let mut serial_buf = [0];

    while let Ok(readed_size) = port.read(&mut serial_buf) {
        if readed_size == 0 {
            break;
        }
        readed.push(serial_buf[0]);
    }

    let mut splited = vec![vec![]];
    for b in readed {
        splited.last_mut().unwrap().push(b);
        if b == SM_7_BIT_END_BYTE {
            splited.push(vec![]);
        }
    }

    if splited.last().unwrap().is_empty() {
        splited.pop();
    }

    splited
}

fn reader(port: &mut Box<dyn SerialPort>) -> Vec<WRCPacket> {
    let readed = read_packet(port);
    if readed.is_empty() {
        return vec![];
    }
    // debug!("readed: {readed:02X?}");

    let parsed = readed
        .into_iter()
        .filter_map(|splice| match sm7bits::decode(&splice) {
            Ok((controlbits, decoded)) => {
                // debug!("decoded: {splice:02X?}");
                match controlbits {
                    SM7BitControlBits::USBLocal => {
                        USBLocalPacket::try_from(decoded).ok().map(Either::Left)
                    }
                    SM7BitControlBits::WRC => WRCPacket::try_from(decoded).ok().map(Either::Right),
                }
            }
            Err(e) => {
                error!("Cannot decode: {splice:02X?}, reason: {e}");
                None
            }
        })
        .collect::<Vec<_>>();

    // 如果USBLOCAL和WRC是配对的包可以用如下方法组合
    // let mut readed_unpair = None;
    // let mut ret = vec![];

    // for r in parsed {
    //     if r.is_left() {
    //         readed_unpair = Some(r);
    //     } else if readed_unpair.is_some() && r.is_right() {
    //         let left = readed_unpair.unwrap().left().unwrap();
    //         let right = r.right().unwrap();
    //         ret.push((left, right));
    //         readed_unpair = None;
    //     }
    // }

    // 目前需求不需要USBLOCAL包，剔除
    parsed.into_iter().filter_map(|r| r.right()).collect()
}

fn com_process<'a>(
    exit_required: Arc<AtomicBool>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    tx: mpsc::Sender<ResponseAction>,
    mut rx: BusReader<RequiredAction>,
) -> anyhow::Result<()> {
    let mut port = serialport::new(port, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()?;

    let mut connection_pending: Vec<WrenchInfo> = vec![];
    let mut mac_to_serial: HashMap<u32, u128> = HashMap::new();
    let mut serial_to_mac: HashMap<u128, u32> = HashMap::new();
    let mut serial_to_name: HashMap<u128, String> = HashMap::new();
    let mut name_to_serial: HashMap<String, u128> = HashMap::new();
    let mut last_heart_beat: HashMap<u32, Instant> = HashMap::new();
    let mut last_seqid: HashMap<u32, u16> = HashMap::new();

    while !exit_required.load(Ordering::Acquire) {
        let readed = reader(&mut port);

        let mut visited = HashSet::new();
        for wrc in readed {
            last_heart_beat.insert(wrc.mac, Instant::now());
            debug!(
                "Mac: {:X?} LastHeartBeat: {:X?}",
                wrc.mac, last_heart_beat[&wrc.mac]
            );

            if !mac_to_serial.contains_key(&wrc.mac) && !visited.contains(&wrc.mac) {
                visited.insert(wrc.mac);
                let query_packet = WRCPacket {
                    sequence_id: 0,
                    mac: wrc.mac,
                    flag: WRCPacketFlag(25),
                    payload_len: std::mem::size_of::<WRCPayloadGetInfo>() as u8,
                    payload: WRCPayload::GetInfo(WRCPayloadGetInfo {
                        flag: WRCPayloadGetInfoFlag(1),
                    }),
                };
                let bytes: Vec<u8> = query_packet.try_into().unwrap();
                debug!("Sending serial requesting message by mac: {:X?}", wrc.mac);
                port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;
            }

            if let WRCPayload::InfoSerial(info_serial) = wrc.payload {
                debug!(
                    "Mac: {:X?} Serial: {:X?}",
                    wrc.mac,
                    u128::from_le_bytes(info_serial.serial)
                );
                mac_to_serial.insert(wrc.mac, u128::from_le_bytes(info_serial.serial));
                serial_to_mac.insert(u128::from_le_bytes(info_serial.serial), wrc.mac);
                continue;
            }

            if let WRCPayload::InfoGeneric(info_generic) = wrc.payload {
                debug!(
                    "Mac: {:X?} LastSeqId: {:X?}",
                    wrc.mac, info_generic.last_server_packet_seqid
                );
                last_seqid.insert(wrc.mac, info_generic.last_server_packet_seqid);
                continue;
            }
        }

        connection_pending = connection_pending
            .into_iter()
            .filter_map(|mut w| {
                for i in serial_to_mac.iter() {
                    if !serial_to_name.contains_key(i.0) {
                        name_to_serial.insert(w.connect_id.clone(), *i.0);
                        serial_to_name.insert(*i.0, w.connect_id.clone());
                        w.wrench_serial = *i.0;
                        tx.send(ResponseAction::BindStatus(w)).unwrap();
                        return None;
                    }
                }

                Some(w)
            })
            .collect();

        match rx.try_recv() {
            Ok(RequiredAction::BindWrench(target)) => {
                connection_pending.push(target);
            }
            Ok(RequiredAction::CheckConnect(mut target)) => {
                target.status = false;

                if !serial_to_mac.contains_key(&target.wrench_serial) {
                    tx.send(ResponseAction::ConnectStatus(target))?;
                    continue;
                }
                let mac = serial_to_mac[&target.wrench_serial];

                if !last_heart_beat.contains_key(&mac) {
                    tx.send(ResponseAction::ConnectStatus(target))?;
                    continue;
                }
                let last_hb = last_heart_beat[&mac];

                if last_hb.elapsed() < Duration::from_secs(30) {
                    target.status = true;
                }

                tx.send(ResponseAction::ConnectStatus(target))?;
            }
            _ => {}
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
        if let Err(e) = com_process(exit_required, port, tx, rx) {
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
