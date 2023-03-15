use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex,
    },
    time::{Duration, Instant},
};

use crate::{
    hardware::{message::wrc::WRCPacket, sm7bits::SM7BitControlBits},
    message::Action,
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
                    SM7BitControlBits::USBLocal => USBLocalPacket::try_from(decoded)
                        .ok()
                        .map(|pkt| Either::Left(pkt)),
                    SM7BitControlBits::WRC => WRCPacket::try_from(decoded)
                        .ok()
                        .map(|pkt| Either::Right(pkt)),
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
    tx: mpsc::Sender<Action>,
    mut rx: BusReader<Action>,
) -> anyhow::Result<()> {
    let mut port = serialport::new(port, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()?;

    let mut mac_to_serial: HashMap<u32, u128> = HashMap::new();
    let mut serial_to_mac: HashMap<u128, u32> = HashMap::new();
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
                port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;
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

        if let Ok(Action::CheckConnect(target)) = rx.try_recv() {
            let mut connected = false;
            if let Some(mac) = serial_to_mac.get(&target.wrench_serial) {
                if let Some(last_heart_beat) = last_heart_beat.get(&mac) {
                    if last_heart_beat.elapsed() < Duration::from_secs(30) {
                        connected = true;
                    }
                }
            }
            tx.send(Action::ConnectStatus((connected, target)))?;
        }
    }

    Ok(())
}

pub fn loop_query(
    exit_required: Arc<AtomicBool>,
    tx: mpsc::Sender<Action>,
    bus: Arc<Mutex<Bus<Action>>>,
) -> anyhow::Result<()> {
    let mut last_com_threads = HashMap::new();
    let mut com_thread_handles = vec![];

    while !exit_required.load(Ordering::Acquire) {
        let ports = serialport::available_ports()?;
        for p in ports.iter() {
            if !last_com_threads.contains_key(&p.port_name) {
                debug!("New port: {}", p.port_name);
                let port = p.port_name.clone();
                {
                    let exit_required = exit_required.clone();
                    let tx = tx.clone();
                    let rx = bus
                        .lock()
                        .map_err(|err| anyhow::anyhow!(err.to_string()))?
                        .add_rx();
                    com_thread_handles.push((
                        port.clone(),
                        std::thread::spawn(move || {
                            if let Err(e) = com_process(exit_required, port, tx, rx) {
                                debug!("Error: {e}");
                            }
                        }),
                    ));
                }
            }
        }
        last_com_threads.clear();
        last_com_threads.extend(ports.into_iter().map(|p| (p.port_name, ())));
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    for (s, h) in com_thread_handles {
        h.join()
            .map_err(|_| anyhow::anyhow!(format!("cannot wait thread handle {:?}", s)))?;
    }

    Ok(())
}
