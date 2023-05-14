use std::collections::hash_map;
use std::sync::mpsc;

use tracing::{debug, info};

use crate::hardware::com_process::ComProcess;

use crate::hardware::com_process::wrench::WrenchContext;
use crate::hardware::message::wrc::{
    WRCPacket, WRCPacketFlag, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag,
};
use crate::message::ResponseAction;

pub fn query_serial(mac: u32, sender: &mpsc::Sender<WRCPacket>) -> anyhow::Result<()> {
    let mut flag = WRCPacketFlag(0);
    flag.set_direction(true);
    flag.set_type(6);
    let mut payload_flag = WRCPayloadGetInfoFlag(0);
    payload_flag.set_serial(true);
    let query_packet = WRCPacket {
        sequence_id: 0,
        mac,
        flag,
        payload_len: 1u8,
        payload: WRCPayload::GetInfo(WRCPayloadGetInfo { flag: payload_flag }),
    };

    sender.send(query_packet)?;

    Ok(())
}

pub fn query_energy(mac: u32, sender: &mpsc::Sender<WRCPacket>) -> anyhow::Result<()> {
    let mut flag = WRCPacketFlag(0);
    flag.set_direction(true);
    flag.set_type(6);
    let mut payload_flag = WRCPayloadGetInfoFlag(0);
    payload_flag.set_energy(true);
    let query_packet = WRCPacket {
        sequence_id: 0,
        mac,
        flag,
        payload_len: 1u8,
        payload: WRCPayload::GetInfo(WRCPayloadGetInfo { flag: payload_flag }),
    };

    sender.send(query_packet)?;

    Ok(())
}

fn verify_mac_serial(
    com: &mut ComProcess,
    wrc: &WRCPacket,
    tx: &mpsc::Sender<ResponseAction>,
) -> bool {
    if let Some(idx) = com.wrenches_mac_map.get(&wrc.mac) {
        if let Some(wrench) = com.wrenches.get_mut(*idx) {
            if let Some(serial_idx) = com.wrenches_serial_map.get(&wrench.serial) {
                if serial_idx == idx {
                    wrench.com_update(wrc, tx);
                    return true;
                }
            }
        }
    }

    false
}

pub fn process_com_message(
    com: &mut ComProcess,
    wrc: &WRCPacket,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    debug!("收到串口消息: {:X?}", wrc);
    if let WRCPayload::InfoSerial(ref info_serial) = wrc.payload {
        let serial = u128::from_le_bytes(info_serial.serial);
        if serial == 0 {
            return Ok(());
        }
        if let hash_map::Entry::Vacant(e) = com.wrenches_serial_map.entry(serial) {
            info!("新扳手 {:X} 上线绑定到Mac: {:X}", serial, wrc.mac);
            let idx = com.wrenches.len();
            e.insert(idx);
            com.wrenches_mac_map.insert(wrc.mac, idx);
            com.wrenches.push(WrenchContext::new(wrc.mac, serial));
            query_energy(wrc.mac, &com.writer)?;
        } else {
            info!("扳手 {:X} 迁移到Mac: {:X}", serial, wrc.mac);
            let idx = *com.wrenches_serial_map.get(&serial).unwrap();
            com.wrenches[idx].mac_reconnect(wrc.mac, &com.writer, tx);
            com.wrenches_mac_map.insert(wrc.mac, idx);
            query_energy(wrc.mac, &com.writer)?;
        }
    } else if !verify_mac_serial(com, wrc, tx) {
        info!("不匹配的Mac: {:X}, 重新查询序列号", wrc.mac);
        query_serial(wrc.mac, &com.writer)?;
    }

    Ok(())
}
