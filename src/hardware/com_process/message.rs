use std::{sync::mpsc, time::Instant};

use tracing::debug;

use crate::hardware::com_process::{parse_float, ComProcess};

use crate::hardware::message::wrc::{
    WRCPacket, WRCPacketFlag, WRCPacketType, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag,
    WRCPayloadInlineJointData,
};
use crate::message::{FinishedInfo, ResponseAction};

pub fn query_serial(mac: u32, sender: &mpsc::Sender<WRCPacket>) -> anyhow::Result<()> {
    let query_packet = WRCPacket {
        sequence_id: 0,
        mac,
        flag: WRCPacketFlag(25),
        payload_len: 1u8,
        payload: WRCPayload::GetInfo(WRCPayloadGetInfo {
            flag: WRCPayloadGetInfoFlag(1),
        }),
    };

    sender.send(query_packet)?;

    Ok(())
}

fn parse_to_float(mut int: i32, mut scale: i32) -> String {
    let mut frac = 0;
    let mut level = 0;

    while scale > 0 {
        frac += i32::pow(10, level) * (int % 10);
        int /= 10;
        scale -= 1;
        level += 1;
    }

    format!("{}.{}", int, frac.abs())
}

pub fn process_com_message(
    com: &mut ComProcess,
    wrc: &WRCPacket,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
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
        query_serial(wrc.mac, &com.writer)?;
        com.data
            .mac_to_seqid_list
            .entry(wrc.mac)
            .or_insert(vec![(0, WRCPacketType::GetInfo)]);
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
            process_inline_joint_data(com, wrc, inline_joint_data, tx)?;
        }
    }

    Ok(())
}

fn process_inline_joint_data(
    com: &mut ComProcess,
    wrc: &WRCPacket,
    inline_joint_data: &[WRCPayloadInlineJointData],
    tx: &mpsc::Sender<ResponseAction>,
) -> Result<(), anyhow::Error> {
    let target = com.data.mac_to_joints.entry(wrc.mac).or_insert(vec![]);
    for recv in inline_joint_data.iter() {
        if target.iter().any(|x| x.joint_id == recv.joint_id) {
            continue;
        }

        target.push(recv.clone());

        match com.data.mac_to_tasks.get(&wrc.mac) {
            Some(pending) if !pending.finished && pending.current_task_id == recv.task_id => {
                let com_task = pending.tasks[pending.current as usize].clone();
                let wrench_serial =
                    u128::from_str_radix(&com_task.request.wrench_serial, 16).unwrap_or_default();
                let torque = parse_float(&com_task.request.torque, 3).unwrap_or_default();
                let torque_up =
                    parse_float(&com_task.request.torque_deviation_up, 3).unwrap_or_default();
                let torque_down =
                    parse_float(&com_task.request.torque_deviation_down, 3).unwrap_or_default();
                let torque_range = (torque - torque_down)..=(torque + torque_up);
                let angle = parse_float(&com_task.request.angle, 1).unwrap_or_default();
                let angle_up =
                    parse_float(&com_task.request.angle_deviation_up, 1).unwrap_or_default();
                let angle_down =
                    parse_float(&com_task.request.angle_deviation_down, 1).unwrap_or_default();
                let angle_range = (angle - angle_down)..=(angle + angle_up);
                let status = if com_task.request.work_mode == "0" {
                    torque_range.contains(&recv.torque)
                } else if com_task.request.work_mode == "1" {
                    angle_range.contains(&(recv.angle as i32))
                } else {
                    torque_range.contains(&recv.torque)
                        && angle_range.contains(&(recv.angle as i32))
                };

                tx.send(ResponseAction::TaskFinished(FinishedInfo {
                    msg_id: com_task.msg_id,
                    wrench_serial,
                    task_id: com_task.request.task_id,
                    task_detail_id: com_task.request.task_detail_id,
                    torque: parse_to_float(recv.torque, 3),
                    angle: parse_to_float(recv.angle as i32, 1),
                    status,
                    start_date: com_task.startup_time,
                    end_date: chrono::Local::now(),
                }))?;
            }
            _ => (),
        }
    }

    Ok(())
}
