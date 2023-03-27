mod message;
mod port;
mod redis;

use std::{
    collections::HashMap,
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

use tracing::{debug, error, info, span, Level};

use crate::{
    hardware::message::wrc::{WRCPacketFlag, WRCPayload},
    message::{RequiredAction, ResponseAction, WrenchInfo},
    redis::message::TaskRequestMsg,
};

use self::{
    message::{process_com_message, query_serial},
    port::read_write_loop,
    redis::process_message_from_redis,
};

use super::message::wrc::{
    WRCPacket, WRCPacketType, WRCPayloadGetJointData, WRCPayloadInlineJointData,
    WRCPayloadSetJoint, WRCPayloadSetJointFlag,
};

fn parse_float(s: &str, mut scale: i32) -> anyhow::Result<i32> {
    let mut s = s.split('.');
    let mut int_side = s.next().unwrap_or("0").parse::<i32>()?;
    let mut dec_side = s.next().unwrap_or("0");
    while scale > 0 {
        scale -= 1;
        int_side *= 10;
        if dec_side.is_empty() {
            continue;
        }
        int_side += dec_side
            .chars()
            .next()
            .unwrap_or('0')
            .to_digit(10)
            .unwrap_or(0) as i32;
        dec_side = dec_side.get(1..).unwrap_or("");
    }
    Ok(int_side)
}

fn send_task(
    task: &TaskRequestMsg,
    sequence_id: u16,
    mac: u32,
    sender: &mpsc::Sender<WRCPacket>,
) -> anyhow::Result<()> {
    let torque = {
        match parse_float(&task.torque, 3) {
            Ok(t) => t,
            Err(e) => bail!("无法解析torque: {e}"),
        }
    };
    let torque_angle_start = {
        match parse_float(&task.torque_angle_start, 3) {
            Ok(t) => t,
            Err(e) => bail!("无法解析torque_angle_start: {e}"),
        }
    };
    let torque_upper_tol = {
        match parse_float(&task.torque_deviation_up, 3) {
            Ok(t) => t,
            Err(e) => bail!("无法解析torque_deviation_up: {e}"),
        }
    };
    let torque_lower_tol = {
        match parse_float(&task.torque_deviation_down, 3) {
            Ok(t) => t,
            Err(e) => bail!("无法解析torque_deviation_down: {e}"),
        }
    };
    let angle = {
        match parse_float(&task.angle, 1) {
            Ok(t) => t,
            Err(e) => bail!("无法解析angle: {e}"),
        }
    };
    let angle_upper_tol = {
        match parse_float(&task.angle_deviation_up, 1) {
            Ok(t) => t,
            Err(e) => bail!("无法解析angle_deviation_up: {e}"),
        }
    };
    let angle_lower_tol = {
        match parse_float(&task.angle_deviation_down, 1) {
            Ok(t) => t,
            Err(e) => bail!("无法解析angle_deviation_down: {e}"),
        }
    };
    let task_repeat_times = { task.bolt_num.parse::<u16>()? };
    let task_id = { task.task_id.parse::<u16>()? };
    let mut task_flag = WRCPayloadSetJointFlag(0);
    task_flag.set_mode(task.work_mode.parse::<u8>()?);
    task_flag.set_method(task.control_mode.parse::<u8>()?);
    task_flag.set_unit(task.unit.parse::<u8>()?);

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
            angle: angle as i16,
            angle_upper_tol: angle_upper_tol as i16,
            angle_lower_tol: angle_lower_tol as i16,
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

#[derive(Debug, Clone)]
pub struct ComTask {
    pub startup_time: chrono::DateTime<chrono::Local>,
    pub msg_id: String,
    pub request: TaskRequestMsg,
}

#[derive(Debug, Clone)]
pub struct PendingTask {
    pub finished: bool,
    pub current: i32,
    pub current_task_id: u16,
    pub tasks: Vec<ComTask>,
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

fn drop_old_heart_beat(com: &mut ComProcess) -> anyhow::Result<()> {
    for (mac, lhb) in com.data.last_heart_beat.iter() {
        if lhb.elapsed() >= Duration::from_secs(30) {
            query_serial(*mac, &com.writer)?;
        }
    }
    com.data
        .last_heart_beat
        .retain(|_, v| v.elapsed() <= Duration::from_secs(35));

    Ok(())
}

fn bind_wrench(com: &mut ComProcess, tx: &mpsc::Sender<ResponseAction>) -> anyhow::Result<()> {
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

    Ok(())
}

fn update_task_status(com: &mut ComProcess) -> anyhow::Result<()> {
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

        if let Some(com_task) = task.tasks.get_mut(task.current as usize) {
            let torque = parse_float(&com_task.request.torque, 3).unwrap_or_default();
            let torque_up =
                parse_float(&com_task.request.torque_deviation_up, 3).unwrap_or_default();
            let torque_down =
                parse_float(&com_task.request.torque_deviation_down, 3).unwrap_or_default();
            let torque_range = (torque - torque_down)..=(torque + torque_up);
            let angle = parse_float(&com_task.request.angle, 1).unwrap_or_default();
            let angle_up = parse_float(&com_task.request.angle_deviation_up, 1).unwrap_or_default();
            let angle_down =
                parse_float(&com_task.request.angle_deviation_down, 1).unwrap_or_default();
            let angle_range = (angle - angle_down)..=(angle + angle_up);
            let ok_num = com
                .data
                .mac_to_joints
                .get(mac)
                .map(|joints| {
                    joints
                        .iter()
                        .filter(|j| j.task_id == task.current_task_id)
                        .filter(|j| {
                            if com_task.request.work_mode == "0" {
                                torque_range.contains(&j.torque)
                            } else if com_task.request.work_mode == "1" {
                                angle_range.contains(&(j.angle as i32))
                            } else {
                                torque_range.contains(&j.torque)
                                    && angle_range.contains(&(j.angle as i32))
                            }
                        })
                        .count()
                })
                .unwrap_or_default();
            let limit_num = com_task
                .request
                .bolt_num
                .parse::<usize>()
                .unwrap_or_default();
            if ok_num == limit_num {
                debug!("任务 {} 结束", task.current_task_id);
                task.finished = true;
            }
        }

        if !task.finished {
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
            let is_time_out = com
                .data
                .mac_to_query_timestamp
                .entry(*mac)
                .or_insert(Instant::now())
                .elapsed()
                >= Duration::from_secs(2);

            if joints_num > 0 && is_time_out {
                com.data.mac_to_query_timestamp.insert(*mac, Instant::now());
                match get_joint_data(seqid + 1, *mac, joints_start, joints_num, &com.writer) {
                    Ok(_) => match com.data.mac_to_seqid_list.get_mut(mac) {
                        Some(seqid_list) => {
                            seqid_list.push((seqid + 1, WRCPacketType::GetJointData));
                        }
                        None => {
                            error!("找不到 Mac: {:X} 的 seqid 列表", mac);
                        }
                    },
                    Err(e) => {
                        error!("获取扳手数据失败: {}", e);
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

        let mut com_task = task.tasks[task.current as usize].clone();
        info!(
            "开始执行任务ID: {}, 子ID: {}, 映射到任务ID: {}",
            com_task.request.task_id, com_task.request.task_detail_id, task.current_task_id
        );
        com.data
            .mac_to_task_id_map
            .entry(*mac)
            .or_insert(HashMap::new())
            .insert(task.current_task_id, com_task.request.task_id);
        com_task.request.task_id = task.current_task_id.to_string();
        match send_task(&com_task.request, seqid + 1, *mac, &com.writer) {
            Ok(_) => match com.data.mac_to_seqid_list.get_mut(mac) {
                Some(seqid_list) => {
                    seqid_list.push((seqid + 1, WRCPacketType::SetJoint));
                }
                None => {
                    error!("找不到 Mac: {:X} 的 seqid 列表", mac);
                }
            },
            Err(e) => {
                error!("发送任务失败: {}", e);
            }
        }
    }

    Ok(())
}

fn com_update(com: &mut ComProcess, tx: &mpsc::Sender<ResponseAction>) -> anyhow::Result<()> {
    drop_old_heart_beat(com)?;

    bind_wrench(com, tx)?;

    update_task_status(com)?;

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
            debug!("收到串口消息: {:02X?}", wrc);
            if let Err(e) = process_com_message(&mut com, &wrc, &tx) {
                error!("处理串口消息失败: {}", e);
            }
        }

        if let Ok(action) = rx.try_recv() {
            debug!("收到 Redis 消息: {:02X?}", action);
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
