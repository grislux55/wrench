use std::{
    collections::HashMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

use anyhow::bail;
use bus::BusReader;
use num_bigfloat::BigFloat;
use serialport::SerialPort;
use tracing::{debug, error};

use crate::{
    hardware::message::wrc::{WRCPacketFlag, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag},
    message::{ConnectInfo, RequiredAction, ResponseAction, TaskInfo, WrenchInfo},
    redis::message::TaskRequestMsg,
};

use super::{
    message::wrc::{
        WRCPacket, WRCPacketType, WRCPayloadGetJointData, WRCPayloadInlineJointData,
        WRCPayloadSetJoint, WRCPayloadSetJointFlag,
    },
    sm7bits::{self, SM7BitControlBits, SM_7_BIT_END_BYTE},
};

fn read_packet(
    exit_required: Arc<AtomicBool>,
    port: &mut Box<dyn serialport::SerialPort>,
) -> Vec<u8> {
    let mut readed = vec![];
    let mut serial_buf = [0];

    while !exit_required.load(Ordering::Acquire) {
        match port.read(&mut serial_buf) {
            Ok(readed_size) => {
                if readed_size == 0 {
                    break;
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
            Err(_) => {
                break;
            }
        }
    }

    readed
}

fn reader(exit_required: Arc<AtomicBool>, port: &mut Box<dyn SerialPort>) -> Option<WRCPacket> {
    let readed = read_packet(exit_required, port);
    if readed.is_empty() {
        return None;
    }
    // debug!("readed: {readed:02X?}");

    match sm7bits::decode(&readed) {
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
    }
}

fn query_serial(wrc: &WRCPacket, port: &mut Box<dyn SerialPort>) -> anyhow::Result<()> {
    let query_packet = WRCPacket {
        sequence_id: 0,
        mac: wrc.mac,
        flag: WRCPacketFlag(25),
        payload_len: 1u8,
        payload: WRCPayload::GetInfo(WRCPayloadGetInfo {
            flag: WRCPayloadGetInfoFlag(1),
        }),
    };
    let bytes: Vec<u8> = query_packet.try_into().unwrap();
    debug!("Sending serial requesting message by mac: {:X?}", wrc.mac);
    port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;
    Ok(())
}

fn clear_task(seqid: u16, mac: u32, port: &mut Box<dyn SerialPort>) -> anyhow::Result<()> {
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
    let bytes: Vec<u8> = clear_packet.try_into().unwrap();
    debug!("Sending clear task message by mac: {:X?}", mac);
    port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;

    Ok(())
}

fn send_task(
    task: &TaskRequestMsg,
    sequence_id: u16,
    mac: u32,
    port: &mut Box<dyn SerialPort>,
) -> anyhow::Result<()> {
    debug!("Sending task to mac: {mac:08X}", mac = mac);
    let torque = {
        if task.torque.is_none() {
            bail!("Torque is not set");
        }
        match BigFloat::from_str(task.torque.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("Cannot parse torque: {e}"),
        }
    };
    let torque_angle_start = {
        if task.torque_angle_start.is_none() {
            bail!("Torque angle start is not set");
        }
        match BigFloat::from_str(task.torque_angle_start.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("Cannot parse torque angle start: {e}"),
        }
    };
    let torque_upper_tol = {
        if task.torque_deviation_up.is_none() {
            bail!("Torque upper tolerance is not set");
        }
        match BigFloat::from_str(task.torque_deviation_up.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("Cannot parse torque upper tolerance: {e}"),
        }
    };
    let torque_lower_tol = {
        if task.torque_deviation_down.is_none() {
            bail!("Torque lower tolerance is not set");
        }
        match BigFloat::from_str(task.torque_deviation_down.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(1000);
                t.to_i128().unwrap() as i32
            }
            Err(e) => bail!("Cannot parse torque lower tolerance: {e}"),
        }
    };
    let angle = {
        if task.angle.is_none() {
            bail!("Angle is not set");
        }
        match BigFloat::from_str(task.angle.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("Cannot parse angle: {e}"),
        }
    };
    let angle_upper_tol = {
        if task.angle_deviation_up.is_none() {
            bail!("Angle upper tolerance is not set");
        }
        match BigFloat::from_str(task.angle_deviation_up.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("Cannot parse angle upper tolerance: {e}"),
        }
    };
    let angle_lower_tol = {
        if task.angle_deviation_down.is_none() {
            bail!("Angle lower tolerance is not set");
        }
        match BigFloat::from_str(task.angle_deviation_down.as_ref().unwrap()) {
            Ok(mut t) => {
                t *= BigFloat::from(10);
                t.to_i128().unwrap() as i16
            }
            Err(e) => bail!("Cannot parse angle lower tolerance: {e}"),
        }
    };
    let task_repeat_times = {
        if task.bolt_num.is_none() {
            bail!("Task bolt num is not set");
        }
        task.bolt_num.as_ref().unwrap().parse::<u16>()?
    };
    let task_id = {
        if task.task_id.is_none() {
            bail!("Task id is not set");
        }
        task.task_id.as_ref().unwrap().parse::<u16>()?
    };
    let mut task_flag = WRCPayloadSetJointFlag(0);
    if task.work_mode.is_none() {
        bail!("Work mode is not set");
    }
    task_flag.set_mode(task.work_mode.as_ref().unwrap().parse::<u8>()?);
    if task.control_mode.is_none() {
        bail!("Control mode is not set");
    }
    task_flag.set_method(task.control_mode.as_ref().unwrap().parse::<u8>()?);
    if task.unit.is_none() {
        bail!("Unit is not set");
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
    // debug!("{:?}", task_packet);

    let bytes: Vec<u8> = task_packet.try_into().unwrap();
    // debug!("{:02X?}", bytes);
    port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;

    Ok(())
}

fn get_joint_data(
    sequence_id: u16,
    mac: u32,
    joint_id_start: u16,
    joint_count: u8,
    port: &mut Box<dyn SerialPort>,
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
    // debug!("{:?}", get_joint_packet);

    let bytes: Vec<u8> = get_joint_packet.try_into().unwrap();
    // debug!("{:02X?}", bytes);
    port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;

    Ok(())
}

fn check_connect(
    mut target: ConnectInfo,
    serial_to_mac: &HashMap<u128, u32>,
    last_heart_beat: &HashMap<u32, Instant>,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    target.status = false;

    if !serial_to_mac.contains_key(&target.wrench_serial) {
        tx.send(ResponseAction::ConnectStatus(target))?;
        return Ok(());
    }
    let mac = serial_to_mac[&target.wrench_serial];

    if !last_heart_beat.contains_key(&mac) {
        tx.send(ResponseAction::ConnectStatus(target))?;
        return Ok(());
    }
    let last_hb = last_heart_beat[&mac];

    if last_hb.elapsed() < Duration::from_secs(30) {
        target.status = true;
    }

    tx.send(ResponseAction::ConnectStatus(target))?;

    Ok(())
}

pub struct PendingTask {
    pub finished: bool,
    pub current: i32,
    pub current_task_id: u16,
    pub tasks: Vec<TaskRequestMsg>,
}

#[allow(clippy::too_many_arguments)]
fn action_send_task(
    msg_id: String,
    target: Vec<TaskRequestMsg>,
    tx: &mpsc::Sender<ResponseAction>,
    serial_to_mac: &HashMap<u128, u32>,
    mac_to_tasks: &mut HashMap<u32, PendingTask>,
    mac_to_seqid_list: &HashMap<u32, Vec<(u16, WRCPacketType)>>,
    mac_to_joint_num: &mut HashMap<u32, u16>,
    port: &mut Box<dyn SerialPort>,
) -> anyhow::Result<()> {
    let mut task_info = TaskInfo {
        msg_id,
        wrench_serial: 0,
        status: false,
    };

    if target.is_empty() {
        error!("empty task");
        tx.send(ResponseAction::TaskStatus(task_info))?;
        return Ok(());
    }

    if target[0].wrench_serial.is_none() {
        error!("serial number should not be None");
        tx.send(ResponseAction::TaskStatus(task_info))?;
        return Ok(());
    }

    let wrench_serial = match u128::from_str_radix(target[0].wrench_serial.as_ref().unwrap(), 16) {
        Ok(s) => s,
        Err(_) => {
            error!("invalid serial number");
            tx.send(ResponseAction::TaskStatus(task_info))?;
            return Ok(());
        }
    };
    task_info.wrench_serial = wrench_serial;

    let mac = match serial_to_mac.get(&wrench_serial) {
        Some(&m) => m,
        None => {
            error!("unknown serial number");
            tx.send(ResponseAction::TaskStatus(task_info))?;
            return Ok(());
        }
    };

    mac_to_tasks
        .entry(mac)
        .and_modify(|pending_task| {
            pending_task.tasks.extend_from_slice(&target);
        })
        .or_insert_with(|| {
            if let Some(&(seqid, _)) = mac_to_seqid_list.get(&mac).and_then(|x| x.last()) {
                if let Err(e) = clear_task(seqid, mac, port) {
                    error!("clear task failed: {}", e);
                }
            } else {
                error!("no seqid found, task will not be cleared");
            }
            mac_to_joint_num.insert(mac, 0);
            PendingTask {
                finished: true,
                current: -1,
                current_task_id: 0,
                tasks: target,
            }
        });

    task_info.status = true;
    tx.send(ResponseAction::TaskStatus(task_info))?;

    Ok(())
}

pub fn com_process<'a>(
    exit_required: Arc<AtomicBool>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    tx: mpsc::Sender<ResponseAction>,
    mut rx: BusReader<RequiredAction>,
) -> anyhow::Result<()> {
    let mut port = {
        let port = port.into();
        loop {
            if exit_required.load(Ordering::Acquire) {
                return Ok(());
            }
            if let Ok(p) = serialport::new(port.clone(), 115_200)
                .timeout(Duration::from_millis(1000))
                .open()
            {
                break p;
            } else {
                error!("Cannot open port {}, will retry after 1 sec", port);
                std::thread::sleep(Duration::from_millis(1000));
            }
        }
    };

    let mut connection_pending: Vec<WrenchInfo> = vec![];
    let mut mac_to_serial: HashMap<u32, u128> = HashMap::new();
    let mut serial_to_mac: HashMap<u128, u32> = HashMap::new();
    let mut serial_to_name: HashMap<u128, String> = HashMap::new();
    let mut name_to_serial: HashMap<String, u128> = HashMap::new();
    let mut last_heart_beat: HashMap<u32, Instant> = HashMap::new();
    let mut mac_to_seqid_list: HashMap<u32, Vec<(u16, WRCPacketType)>> = HashMap::new();
    let mut mac_to_tasks: HashMap<u32, PendingTask> = HashMap::new();
    let mut mac_to_joints: HashMap<u32, Vec<WRCPayloadInlineJointData>> = HashMap::new();
    let mut mac_to_joint_num: HashMap<u32, u16> = HashMap::new();
    let mut mac_to_query_timestamp: HashMap<u32, Instant> = HashMap::new();
    let mut mac_to_task_id_map: HashMap<u32, HashMap<u16, String>> = HashMap::new();

    while !exit_required.load(Ordering::Acquire) {
        let readed = reader(exit_required.clone(), &mut port);

        if let Some(wrc) = readed {
            last_heart_beat.insert(wrc.mac, Instant::now());
            // debug!(
            //     "Mac: {:X?} LastHeartBeat: {:X?}",
            //     wrc.mac, last_heart_beat[&wrc.mac]
            // );
            if let WRCPayload::InfoSerial(ref info_serial) = wrc.payload {
                debug!(
                    "Mac: {:X?} Serial: {:X?}",
                    wrc.mac,
                    u128::from_le_bytes(info_serial.serial)
                );
                mac_to_serial.insert(wrc.mac, u128::from_le_bytes(info_serial.serial));
                serial_to_mac.insert(u128::from_le_bytes(info_serial.serial), wrc.mac);
            }

            if !mac_to_serial.contains_key(&wrc.mac) {
                if let Err(e) = query_serial(&wrc, &mut port) {
                    error!("query_serial failed: {}", e);
                } else {
                    mac_to_seqid_list
                        .entry(wrc.mac)
                        .or_insert(vec![])
                        .push((0, WRCPacketType::GetInfo));
                }
                continue;
            }

            match wrc.payload {
                WRCPayload::InfoSerial(_) => {}
                WRCPayload::InfoGeneric(ref info_generic) => {
                    debug!(
                        "Mac: {:X?} LastSeqId: {:X?}",
                        wrc.mac, info_generic.last_server_packet_seqid
                    );
                    mac_to_joint_num.insert(wrc.mac, info_generic.joint_count);
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
                WRCPayload::StatusReport(ref status_report) => {
                    debug!("{:?}", status_report);
                }
                WRCPayload::InlineJointData(ref inline_joint_data) => {
                    debug!("{:?}", inline_joint_data);
                    let target = mac_to_joints.entry(wrc.mac).or_insert(vec![]);
                    for recv in inline_joint_data.iter() {
                        if target.iter().any(|x| x.joint_id == recv.joint_id) {
                            continue;
                        }
                        target.push(recv.clone());
                    }
                }
            }
        }

        connection_pending = connection_pending
            .into_iter()
            .filter_map(|mut w| {
                if name_to_serial.contains_key(&w.connect_id) {
                    w.wrench_serial = name_to_serial[&w.connect_id];
                    tx.send(ResponseAction::BindResponse(w)).unwrap();
                    return None;
                }

                for i in serial_to_mac.iter() {
                    if !serial_to_name.contains_key(i.0) {
                        name_to_serial.insert(w.connect_id.clone(), *i.0);
                        serial_to_name.insert(*i.0, w.connect_id.clone());
                        w.wrench_serial = *i.0;
                        tx.send(ResponseAction::BindResponse(w)).unwrap();
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
            Ok(RequiredAction::CheckConnect(target)) => {
                if let Err(e) = check_connect(target, &serial_to_mac, &last_heart_beat, &tx) {
                    error!("check_connect failed: {}", e);
                }
            }
            Ok(RequiredAction::SendTask((msg_id, target))) => {
                if let Err(e) = action_send_task(
                    msg_id,
                    target,
                    &tx,
                    &serial_to_mac,
                    &mut mac_to_tasks,
                    &mac_to_seqid_list,
                    &mut mac_to_joint_num,
                    &mut port,
                ) {
                    error!("action_send_task failed: {}", e);
                }
            }
            Err(_) => {}
        }

        for (mac, task) in mac_to_tasks.iter_mut() {
            if task.current + 1 >= task.tasks.len() as i32 && task.finished {
                continue;
            }

            let seqid = match mac_to_seqid_list.get(mac).and_then(|x| x.last()) {
                Some(&(s, _)) => s,
                None => {
                    error!("last seqid not found");
                    continue;
                }
            };

            if let Some(t) = task.tasks.get_mut(task.current as usize) {
                let ok_num = mac_to_joints
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
                    debug!("task {} finished", task.current_task_id);
                    task.finished = true;
                }
            }

            let joints_start = mac_to_joints
                .get(mac)
                .map(|x| x.len() as u16)
                .unwrap_or_default();
            let joints_num = mac_to_joint_num
                .get(mac)
                .cloned()
                .unwrap_or_default()
                .saturating_sub(joints_start) as u8;

            if !task.finished {
                if joints_num > 0
                    && mac_to_query_timestamp
                        .entry(*mac)
                        .or_insert(Instant::now())
                        .elapsed()
                        < Duration::from_secs(5)
                {
                    mac_to_query_timestamp.insert(*mac, Instant::now());
                    debug!("joints_start: {}, joints_num: {}", joints_start, joints_num);
                    if let Err(e) =
                        get_joint_data(seqid + 1, *mac, joints_start, joints_num, &mut port)
                    {
                        error!("get joint data failed: {}", e);
                    } else {
                        match mac_to_seqid_list.get_mut(mac) {
                            Some(seqid_list) => {
                                seqid_list.push((seqid + 1, WRCPacketType::GetJointData));
                            }
                            None => {
                                error!("mac_to_seqid_list not found");
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
            mac_to_task_id_map
                .entry(*mac)
                .or_insert(HashMap::new())
                .insert(
                    task.current_task_id,
                    task_request.task_id.unwrap_or("0".to_string()),
                );
            task_request.task_id = Some(task.current_task_id.to_string());
            if let Err(e) = send_task(&task_request, seqid + 1, *mac, &mut port) {
                error!("send task failed: {}", e);
            } else {
                match mac_to_seqid_list.get_mut(mac) {
                    Some(seqid_list) => {
                        seqid_list.push((seqid + 1, WRCPacketType::SetJoint));
                    }
                    None => {
                        error!("seqid_list for mac {:?} not found", mac);
                    }
                }
            }
        }
    }

    Ok(())
}
