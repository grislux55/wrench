use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

use bus::BusReader;
use serialport::SerialPort;
use tracing::{debug, error};

use crate::{
    hardware::message::wrc::{WRCPacketFlag, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag},
    message::{RequiredAction, ResponseAction, WrenchInfo},
};

use super::{
    message::wrc::WRCPacket,
    sm7bits::{self, SM7BitControlBits, SM_7_BIT_END_BYTE},
};

fn read_packet(
    exit_required: Arc<AtomicBool>,
    port: &mut Box<dyn serialport::SerialPort>,
) -> Vec<u8> {
    let mut readed = vec![];
    let mut serial_buf = [0];

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(readed_size) = port.read(&mut serial_buf) {
            if readed_size == 0 {
                continue;
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
    }

    readed
}

fn reader(exit_required: Arc<AtomicBool>, port: &mut Box<dyn SerialPort>) -> Option<WRCPacket> {
    while !exit_required.load(Ordering::Acquire) {
        let readed = read_packet(exit_required.clone(), port);
        if readed.is_empty() {
            continue;
        }
        // debug!("readed: {readed:02X?}");

        let decoded = match sm7bits::decode(&readed) {
            Ok((SM7BitControlBits::WRC, decoded)) => {
                match WRCPacket::try_from(decoded) {
                    Ok(p) => {
                        // debug!("parsed: {p:?}");
                        Some(p)
                    }
                    Err(e) => {
                        error!("Cannot parse: {readed:02X?}, reason: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                error!("Cannot decode: {readed:02X?}, reason: {e}");
                None
            }
            _ => None,
        };

        if decoded.is_some() {
            return decoded;
        }
    }

    None
}

fn query_serial(wrc: &WRCPacket, port: &mut Box<dyn SerialPort>) -> Result<(), anyhow::Error> {
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
    Ok(())
}

pub fn com_process<'a>(
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
        let readed = reader(exit_required.clone(), &mut port);

        let mut visited = HashSet::new();
        if let Some(wrc) = readed {
            last_heart_beat.insert(wrc.mac, Instant::now());
            debug!(
                "Mac: {:X?} LastHeartBeat: {:X?}",
                wrc.mac, last_heart_beat[&wrc.mac]
            );

            if !mac_to_serial.contains_key(&wrc.mac) && !visited.contains(&wrc.mac) {
                visited.insert(wrc.mac);
                query_serial(&wrc, &mut port)?;
            }

            match wrc.payload {
                WRCPayload::InfoSerial(info_serial) => {
                    debug!(
                        "Mac: {:X?} Serial: {:X?}",
                        wrc.mac,
                        u128::from_le_bytes(info_serial.serial)
                    );
                    mac_to_serial.insert(wrc.mac, u128::from_le_bytes(info_serial.serial));
                    serial_to_mac.insert(u128::from_le_bytes(info_serial.serial), wrc.mac);
                }
                WRCPayload::InfoGeneric(info_generic) => {
                    debug!(
                        "Mac: {:X?} LastSeqId: {:X?}",
                        wrc.mac, info_generic.last_server_packet_seqid
                    );
                    last_seqid.insert(wrc.mac, info_generic.last_server_packet_seqid);
                }
                WRCPayload::InfoTiming(_) => todo!(),
                WRCPayload::InfoEnergy(_) => todo!(),
                WRCPayload::InfoNetwork(_) => todo!(),
                WRCPayload::GetInfo(_) => todo!(),
                WRCPayload::SetJoint(_) => todo!(),
                WRCPayload::SetWrenchTime(_) => todo!(),
                WRCPayload::GetJointData(_) => todo!(),
                WRCPayload::ClearJointData => todo!(),
                WRCPayload::GetStatusReport => todo!(),
                WRCPayload::Beep => todo!(),
                WRCPayload::JointData => todo!(),
                WRCPayload::StatusReport(_) => todo!(),
                WRCPayload::InlineJointData(_) => todo!(),
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
