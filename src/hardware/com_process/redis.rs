use std::{sync::mpsc, time::Duration};

use anyhow::bail;
use tracing::{debug, error};

use crate::{
    hardware::message::wrc::{WRCPacket, WRCPacketFlag, WRCPayload},
    message::{ConnectInfo, RequiredAction, ResponseAction, TaskInfo},
    redis::message::TaskRequestMsg,
};

use super::{ComProcess, PendingTask};

fn check_connect(
    com: &mut ComProcess,
    mut target: ConnectInfo,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    target.status = false;

    if !com.data.serial_to_mac.contains_key(&target.wrench_serial) {
        tx.send(ResponseAction::ConnectStatus(target))?;
        return Ok(());
    }
    let mac = com.data.serial_to_mac[&target.wrench_serial];

    if !com.data.last_heart_beat.contains_key(&mac) {
        tx.send(ResponseAction::ConnectStatus(target))?;
        return Ok(());
    }
    let last_hb = com.data.last_heart_beat[&mac];

    if last_hb.elapsed() < Duration::from_secs(30) {
        target.status = true;
    }

    tx.send(ResponseAction::ConnectStatus(target))?;

    Ok(())
}

fn clear_task(com: &mut ComProcess, seqid: u16, mac: u32) -> anyhow::Result<()> {
    let mut flag = WRCPacketFlag(0);
    flag.set_direction(true);
    flag.set_type(10);
    let clear_packet = WRCPacket {
        sequence_id: seqid,
        mac,
        flag,
        payload_len: 0u8,
        payload: WRCPayload::ClearJointData,
    };
    debug!("Sending clear task message by mac: {:X?}", mac);
    com.writer.send(clear_packet)?;

    Ok(())
}

fn send_task(
    com: &mut ComProcess,
    msg_id: String,
    target: Vec<TaskRequestMsg>,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    let mut task_info = TaskInfo {
        msg_id,
        wrench_serial: 0,
        status: false,
    };

    if target.is_empty() {
        tx.send(ResponseAction::TaskStatus(task_info))?;
        bail!("empty task");
    }

    if target[0].wrench_serial.is_none() {
        tx.send(ResponseAction::TaskStatus(task_info))?;
        bail!("serial number should not be None");
    }

    let wrench_serial = match u128::from_str_radix(target[0].wrench_serial.as_ref().unwrap(), 16) {
        Ok(s) => s,
        Err(_) => {
            tx.send(ResponseAction::TaskStatus(task_info))?;
            bail!("invalid serial number");
        }
    };
    task_info.wrench_serial = wrench_serial;

    let mac = match com.data.serial_to_mac.get(&wrench_serial) {
        Some(&m) => m,
        None => {
            tx.send(ResponseAction::TaskStatus(task_info))?;
            bail!("unknown serial number");
        }
    };

    if com.data.mac_to_tasks.get(&mac).is_none() {
        if let Some(&(seqid, _)) = com.data.mac_to_seqid_list.get(&mac).and_then(|x| x.last()) {
            clear_task(com, seqid, mac)?;
        } else {
            error!("no seqid found, task will not be cleared");
        }
        com.data.mac_to_joint_num.insert(mac, 0);
    }

    com.data
        .mac_to_tasks
        .entry(mac)
        .or_insert(PendingTask {
            finished: true,
            current: -1,
            current_task_id: 0,
            tasks: vec![],
        })
        .tasks
        .extend_from_slice(&target);

    task_info.status = true;
    tx.send(ResponseAction::TaskStatus(task_info))?;

    Ok(())
}

pub fn process_message_from_redis(
    com: &mut ComProcess,
    action: RequiredAction,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    match action {
        RequiredAction::BindWrench(target) => {
            com.data.connection_pending.push(target);
        }
        RequiredAction::CheckConnect(target) => check_connect(com, target, tx)?,
        RequiredAction::SendTask((msg_id, target)) => {
            send_task(com, msg_id, target, tx)?;
        }
    }

    Ok(())
}
