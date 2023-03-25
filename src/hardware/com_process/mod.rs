mod message;
mod port;
mod redis;

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use anyhow::bail;
use bus::BusReader;
use num_bigfloat::BigFloat;

use tracing::{debug, error, info, span, Level};

use crate::{
    hardware::message::wrc::{WRCPacketFlag, WRCPayload},
    message::{RequiredAction, ResponseAction, WrenchInfo},
    redis::message::TaskRequestMsg,
};

use self::{
    message::process_com_message, port::read_write_loop, redis::process_message_from_redis,
};

use super::message::wrc::{
    WRCPacket, WRCPacketType, WRCPayloadGetJointData, WRCPayloadInlineJointData,
    WRCPayloadSetJoint, WRCPayloadSetJointFlag,
};

fn send_task(
    task: &TaskRequestMsg,
    sequence_id: u16,
    mac: u32,
    sender: &mpsc::Sender<WRCPacket>,
) -> anyhow::Result<()> {
    let torque = {
        if task.torque.is_none() {
            bail!("torque不能为空");
        }
        match BigFloat::from_str(task.torque.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("无法解析torque: {e}"),
        }
    };
    let torque_angle_start = {
        if task.torque_angle_start.is_none() {
            bail!("torque_angle_start不能为空");
        }
        match BigFloat::from_str(task.torque_angle_start.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("无法解析torque_angle_start: {e}"),
        }
    };
    let torque_upper_tol = {
        if task.torque_deviation_up.is_none() {
            bail!("torque_deviation_up不能为空");
        }
        match BigFloat::from_str(task.torque_deviation_up.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("无法解析torque_deviation_up: {e}"),
        }
    };
    let torque_lower_tol = {
        if task.torque_deviation_down.is_none() {
            bail!("torque_deviation_down不能为空");
        }
        match BigFloat::from_str(task.torque_deviation_down.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("无法解析torque_deviation_down: {e}"),
        }
    };
    let angle = {
        if task.angle.is_none() {
            bail!("angle不能为空");
        }
        match BigFloat::from_str(task.angle.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("无法解析angle: {e}"),
        }
    };
    let angle_upper_tol = {
        if task.angle_deviation_up.is_none() {
            bail!("angle_deviation_up不能为空");
        }
        match BigFloat::from_str(task.angle_deviation_up.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("无法解析angle_deviation_up: {e}"),
        }
    };
    let angle_lower_tol = {
        if task.angle_deviation_down.is_none() {
            bail!("angle_deviation_down不能为空");
        }
        match BigFloat::from_str(task.angle_deviation_down.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("无法解析angle_deviation_down: {e}"),
        }
    };
    let task_repeat_times = {
        if task.bolt_num.is_none() {
            bail!("bolt_num不能为空");
        }
        task.bolt_num.as_ref().unwrap().parse::<u16>()?
    };
    let task_id = {
        if task.task_id.is_none() {
            bail!("task_id不能为空");
        }
        task.task_id.as_ref().unwrap().parse::<u16>()?
    };
    let mut task_flag = WRCPayloadSetJointFlag(0);
    if task.work_mode.is_none() {
        bail!("work_mode不能为空");
    }
    task_flag.set_mode(task.work_mode.as_ref().unwrap().parse::<u8>()?);
    if task.control_mode.is_none() {
        bail!("control_mode不能为空");
    }
    task_flag.set_method(task.control_mode.as_ref().unwrap().parse::<u8>()?);
    if task.unit.is_none() {
        bail!("unit不能为空");
    }
    task_flag.set_unit(task.unit.as_ref().unwrap().parse::<u8>()?);

    let mut wrc_flag = WRCPacketFlag(0);
    wrc_flag.set_direction(true);
    wrc_flag.set_type(7);
    let task_packet = WRCPacket {
        sequence_id,
        mac,
        flag: wrc_flag,
        payload_len: 33u8,
        payload: WRCPayload::SetJoint(WRCPayloadSetJoint {
            torque_setpoint: torque,
            torque_angle_start,
            torque_upper_tol,
            torque_lower_tol,
            angle,
            angle_upper_tol,
            angle_lower_tol,
            fdt: -1,
            fda: -1,
            task_repeat_times,
            task_id,
            flag: task_flag,
        }),
    };
    sender.send(task_packet)?;

    Ok(())
}

fn get_joint_data(
    sequence_id: u16,
    mac: u32,
    joint_id_start: u16,
    joint_count: u8,
    sender: &mpsc::Sender<WRCPacket>,
) -> anyhow::Result<()> {
    let mut wrc_flag = WRCPacketFlag(0);
    wrc_flag.set_direction(true);
    wrc_flag.set_type(9);
    let get_joint_packet = WRCPacket {
        sequence_id,
        mac,
        flag: wrc_flag,
        payload_len: 3u8,
        payload: WRCPayload::GetJointData(WRCPayloadGetJointData {
            joint_id_start,
            joint_count,
        }),
    };
    sender.send(get_joint_packet)?;

    Ok(())
}

pub struct PendingTask {
    pub finished: bool,
    pub current: i32,
    pub current_task_id: u16,
    pub tasks: Vec<TaskRequestMsg>,
}

#[allow(clippy::too_many_arguments)]

pub struct ComProcess {
    pub reader: Receiver<WRCPacket>,
    pub writer: Sender<WRCPacket>,
    pub handle: JoinHandle<()>,
    pub data: ComProcessData,
}

#[derive(Default)]
pub struct ComProcessData {
    pub connection_pending: Vec<WrenchInfo>,
    pub mac_to_serial: HashMap<u32, u128>,
    pub serial_to_mac: HashMap<u128, u32>,
    pub serial_to_name: HashMap<u128, String>,
    pub name_to_serial: HashMap<String, u128>,
    pub last_heart_beat: HashMap<u32, Instant>,
    pub mac_to_seqid_list: HashMap<u32, Vec<(u16, WRCPacketType)>>,
    pub mac_to_tasks: HashMap<u32, PendingTask>,
    pub mac_to_joints: HashMap<u32, Vec<WRCPayloadInlineJointData>>,
    pub mac_to_joint_num: HashMap<u32, u16>,
    pub mac_to_query_timestamp: HashMap<u32, Instant>,
    pub mac_to_task_id_map: HashMap<u32, HashMap<u16, String>>,
}

fn com_update(com: &mut ComProcess, tx: &mpsc::Sender<ResponseAction>) -> anyhow::Result<()> {
    com.data.connection_pending = com
        .data
        .connection_pending
        .clone()
        .into_iter()
        .filter_map(|mut w| {
            if com.data.name_to_serial.contains_key(&w.connect_id) {
                w.wrench_serial = com.data.name_to_serial[&w.connect_id];
                tx.send(ResponseAction::BindResponse(w)).unwrap();
                return None;
            }

            for i in com.data.serial_to_mac.iter() {
                if !com.data.serial_to_name.contains_key(i.0) {
                    com.data.name_to_serial.insert(w.connect_id.clone(), *i.0);
                    com.data.serial_to_name.insert(*i.0, w.connect_id.clone());
                    w.wrench_serial = *i.0;
                    tx.send(ResponseAction::BindResponse(w)).unwrap();
                    return None;
                }
            }

            Some(w)
        })
        .collect();

    for (mac, task) in com.data.mac_to_tasks.iter_mut() {
        if task.current + 1 >= task.tasks.len() as i32 && task.finished {
            continue;
        }

        let seqid = match com.data.mac_to_seqid_list.get(mac).and_then(|x| x.last()) {
            Some(&(s, _)) => s,
            None => {
                error!("没有找到 Mac: {:X} 的 seqid", mac);
                continue;
            }
        };

        if let Some(t) = task.tasks.get_mut(task.current as usize) {
            let ok_num = com
                .data
                .mac_to_joints
                .get(mac)
                .map(|joints| {
                    joints
                        .iter()
                        .filter(|j| j.task_id == task.current_task_id)
                        .filter(|j| j.flag.is_ok())
                        .count()
                })
                .unwrap_or_default();
            if ok_num
                == t.bolt_num
                    .as_ref()
                    .unwrap_or(&"0".to_string())
                    .parse::<usize>()
                    .unwrap_or_default()
            {
                debug!("任务 {} 结束", task.current_task_id);
                task.finished = true;
            }
        }

        let joints_start = com
            .data
            .mac_to_joints
            .get(mac)
            .map(|x| x.len() as u16)
            .unwrap_or_default();
        let joints_num = com
            .data
            .mac_to_joint_num
            .get(mac)
            .cloned()
            .unwrap_or_default()
            .saturating_sub(joints_start) as u8;

        if !task.finished {
            if joints_num > 0
                && com
                    .data
                    .mac_to_query_timestamp
                    .entry(*mac)
                    .or_insert(Instant::now())
                    .elapsed()
                    < Duration::from_secs(5)
            {
                com.data.mac_to_query_timestamp.insert(*mac, Instant::now());
                if let Err(e) =
                    get_joint_data(seqid + 1, *mac, joints_start, joints_num, &com.writer)
                {
                    error!("获取扳手数据失败: {}", e);
                } else {
                    match com.data.mac_to_seqid_list.get_mut(mac) {
                        Some(seqid_list) => {
                            seqid_list.push((seqid + 1, WRCPacketType::GetJointData));
                        }
                        None => {
                            error!("找不到 Mac: {:X} 的 seqid 列表", mac);
                        }
                    }
                }
            }
            continue;
        }

        if task.current + 1 >= task.tasks.len() as i32 {
            continue;
        }

        task.current += 1;
        task.finished = false;
        task.current_task_id += 1;

        let mut task_request = task.tasks[task.current as usize].clone();
        com.data
            .mac_to_task_id_map
            .entry(*mac)
            .or_insert(HashMap::new())
            .insert(
                task.current_task_id,
                task_request.task_id.unwrap_or("0".to_string()),
            );
        task_request.task_id = Some(task.current_task_id.to_string());
        if let Err(e) = send_task(&task_request, seqid + 1, *mac, &com.writer) {
            error!("发送任务失败: {}", e);
        } else {
            match com.data.mac_to_seqid_list.get_mut(mac) {
                Some(seqid_list) => {
                    seqid_list.push((seqid + 1, WRCPacketType::SetJoint));
                }
                None => {
                    error!("找不到 Mac: {:X} 的 seqid 列表", mac);
                }
            }
        }
    }

    Ok(())
}

pub fn com_process<'a>(
    exit_required: Arc<AtomicBool>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    tx: mpsc::Sender<ResponseAction>,
    mut rx: BusReader<RequiredAction>,
) {
    let port = port.into();
    let mut com = {
        let (thread_writer, reader) = mpsc::channel();
        let (writer, thread_reader) = mpsc::channel();

        let handle = {
            let port = port.to_string();
            let exit_required = exit_required.clone();
            info!("启动串口读写线程");
            std::thread::spawn(move || {
                span!(Level::ERROR, "串口读写线程", port = %port).in_scope(|| {
                    read_write_loop(thread_reader, thread_writer, &port, exit_required);
                });
            })
        };
        ComProcess {
            reader,
            writer,
            handle,
            data: ComProcessData::default(),
        }
    };

    info!("启动处理循环");
    while !exit_required.load(Ordering::Acquire) {
        if let Ok(wrc) = com.reader.try_recv() {
            debug!("收到串口消息: {:?}", wrc);
            if let Err(e) = process_com_message(&mut com, &wrc) {
                error!("处理串口消息失败: {}", e);
            }
        }

        if let Ok(action) = rx.try_recv() {
            debug!("收到 Redis 消息: {:?}", action);
            if let Err(e) = process_message_from_redis(&mut com, action, &tx) {
                error!("处理 Redis 消息失败: {}", e);
            }
        }

        if let Err(e) = com_update(&mut com, &tx) {
            error!("定时更新失败: {}", e);
        }

        std::thread::sleep(Duration::from_secs(1));
    }
}
