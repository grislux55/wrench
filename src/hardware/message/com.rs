use std::{sync::mpsc, time::Instant};

use tracing::debug;

use crate::hardware::com_process::ComProcess;

use super::wrc::{
    WRCPacket, WRCPacketFlag, WRCPacketType, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag,
};

fn query_serial(wrc: &WRCPacket, sender: &mpsc::Sender<WRCPacket>) -> anyhow::Result<()> {
    let query_packet = WRCPacket {
        sequence_id: 0,
        mac: wrc.mac,
        flag: WRCPacketFlag(25),
        payload_len: 1u8,
        payload: WRCPayload::GetInfo(WRCPayloadGetInfo {
            flag: WRCPayloadGetInfoFlag(1),
        }),
    };

    sender.send(query_packet)?;

    Ok(())
}

pub fn process_com_message(com: &mut ComProcess, wrc: &WRCPacket) -> anyhow::Result<()> {
    com.data.last_heart_beat.insert(wrc.mac, Instant::now());

    if let WRCPayload::InfoSerial(ref info_serial) = wrc.payload {
        debug!(
            "Mac: {:X?} Serial: {:X?}",
            wrc.mac,
            u128::from_le_bytes(info_serial.serial)
        );
        com.data
            .mac_to_serial
            .insert(wrc.mac, u128::from_le_bytes(info_serial.serial));
        com.data
            .serial_to_mac
            .insert(u128::from_le_bytes(info_serial.serial), wrc.mac);
        return Ok(());
    }

    if !com.data.mac_to_serial.contains_key(&wrc.mac) {
        query_serial(wrc, &com.writer)?;
        com.data
            .mac_to_seqid_list
            .entry(wrc.mac)
            .or_insert(vec![])
            .push((0, WRCPacketType::GetInfo));
        return Ok(());
    }

    match &wrc.payload {
        WRCPayload::InfoSerial(_) => {}
        WRCPayload::InfoGeneric(info_generic) => {
            com.data
                .mac_to_joint_num
                .insert(wrc.mac, info_generic.joint_count);
        }
        WRCPayload::InfoTiming(_) => {
            debug!("unimplemented!(\"InfoTiming\")");
        }
        WRCPayload::InfoEnergy(_) => {
            debug!("unimplemented!(\"InfoEnergy\")");
        }
        WRCPayload::InfoNetwork(_) => {
            debug!("unimplemented!(\"InfoNetwork\")");
        }
        WRCPayload::GetInfo(_) => {
            debug!("unimplemented!(\"GetInfo\")");
        }
        WRCPayload::SetJoint(_) => {
            debug!("unimplemented!(\"SetJoint\")");
        }
        WRCPayload::SetWrenchTime(_) => {
            debug!("unimplemented!(\"SetWrenchTime\")");
        }
        WRCPayload::GetJointData(_) => {
            debug!("unimplemented!(\"GetJointData\")");
        }
        WRCPayload::ClearJointData => {
            debug!("unimplemented!(\"ClearJointData\")");
        }
        WRCPayload::GetStatusReport => {
            debug!("unimplemented!(\"GetStatusReport\")");
        }
        WRCPayload::Beep => {
            debug!("unimplemented!(\"Beep\")");
        }
        WRCPayload::JointData => {
            debug!("unimplemented!(\"JointData\")");
        }
        WRCPayload::StatusReport(status_report) => {
            debug!("{:?}", status_report);
        }
        WRCPayload::InlineJointData(inline_joint_data) => {
            debug!("{:?}", inline_joint_data);
            let target = com.data.mac_to_joints.entry(wrc.mac).or_insert(vec![]);
            for recv in inline_joint_data.iter() {
                if target.iter().any(|x| x.joint_id == recv.joint_id) {
                    continue;
                }
                target.push(recv.clone());
            }
        }
    }

    Ok(())
}
