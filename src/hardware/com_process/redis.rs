use std::{sync::mpsc, time::Duration};

use anyhow::bail;
use tracing::{debug, error, info};

use crate::{
    hardware::message::wrc::{WRCPacket, WRCPacketFlag, WRCPacketType, WRCPayload},
    message::{ConnectInfo, RequiredAction, ResponseAction, TaskInfo},
    redis::message::TaskRequestMsg,
};

use super::{ComProcess, ComTask, PendingTask};

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

    if last_hb.elapsed() <= Duration::from_secs(35) {
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
    debug!("向Mac地址为: {:X?} 的扳手发送清空任务信号", mac);
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
        msg_id: msg_id.clone(),
        wrench_serial: 0,
        status: false,
    };

    if target.is_empty() {
        tx.send(ResponseAction::TaskStatus(task_info))?;
        bail!("空的任务列表");
    }

    let wrench_serial = match u128::from_str_radix(&target[0].wrench_serial, 16) {
        Ok(s) => s,
        Err(_) => {
            tx.send(ResponseAction::TaskStatus(task_info))?;
            bail!("wrench_serial值不合法");
        }
    };
    task_info.wrench_serial = wrench_serial;

    let mac = match com.data.serial_to_mac.get(&wrench_serial) {
        Some(&m) => m,
        None => {
            tx.send(ResponseAction::TaskStatus(task_info))?;
            bail!("未知的wrench_serial");
        }
    };

    if com.data.mac_to_tasks.get(&mac).is_none() {
        if let Some(&(seqid, _)) = com.data.mac_to_seqid_list.get(&mac).and_then(|x| x.last()) {
            clear_task(com, seqid + 1, mac)?;
            com.data
                .mac_to_seqid_list
                .entry(mac)
                .or_insert(vec![])
                .push((seqid + 1, WRCPacketType::ClearJointData));
        } else {
            error!(
                "找不到Mac地址为: {:X?} 的扳手的seqid, 扳手原有任务不清除",
                mac
            );
        }
        com.data.mac_to_joint_num.insert(mac, 0);
    }

    let local = chrono::Local::now();

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
        .extend(target.iter().map(|x| ComTask {
            msg_id: msg_id.clone(),
            startup_time: local,
            request: x.clone(),
        }));

    task_info.status = true;
    tx.send(ResponseAction::TaskStatus(task_info))?;

    Ok(())
}

fn cancel_task(com: &mut ComProcess, wrench_serial: String, task_id: String) -> anyhow::Result<()> {
    let wrench_serial = match u128::from_str_radix(&wrench_serial, 16) {
        Ok(s) => s,
        Err(_) => {
            bail!("wrench_serial值不合法");
        }
    };
    let mac = match com.data.serial_to_mac.get(&wrench_serial) {
        Some(&m) => m,
        None => {
            bail!("未知的wrench_serial");
        }
    };
    let task = match com.data.mac_to_tasks.get_mut(&mac) {
        Some(t) => t,
        None => {
            bail!("没有任务, 不需要取消任务");
        }
    };

    if task
        .tasks
        .get(task.current as usize)
        .map(|current| &current.request.task_id)
        == Some(&task_id)
    {
        task.finished = true;
    }

    while task.current >= 0 {
        if task
            .tasks
            .get(task.current as usize)
            .map(|x| x.request.task_id == task_id)
            .unwrap_or(true)
        {
            task.current -= 1;
        } else {
            break;
        }
    }

    task.tasks.retain(|x| x.request.task_id != task_id);
    info!(
        "Mac地址为: {:X?} 的扳手的任务列表中删除了任务ID为: {} 的任务",
        mac, task_id
    );
    info!("Mac地址为: {:X?} 的扳手的任务列表如下: {:?}", mac, task);
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
        RequiredAction::TaskCancel((wrench_serial, task_id)) => {
            cancel_task(com, wrench_serial, task_id)?;
        }
    }

    Ok(())
}
