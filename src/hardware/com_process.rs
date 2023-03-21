use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

use anyhow::bail;
use bus::BusReader;
use serialport::SerialPort;
use tracing::{debug, error};

use crate::{
    hardware::message::wrc::{WRCPacketFlag, WRCPayload, WRCPayloadGetInfo, WRCPayloadGetInfoFlag},
    message::{RequiredAction, ResponseAction, TaskInfo, WrenchInfo},
    redis::message::TaskRequestMsg,
};

use super::{
    message::wrc::{WRCPacket, WRCPayloadSetJoint, WRCPayloadSetJointFlag},
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

fn send_task(
    task: &TaskRequestMsg,
    sequence_id: u16,
    mac: u32,
    port: &mut Box<dyn SerialPort>,
) -> anyhow::Result<()> {
    let torque = {
        if task.torque.is_none() {
            bail!("Torque is not set");
        }
        task.torque.as_ref().unwrap().parse::<i32>()?
    };
    let torque_angle_start = {
        if task.torque_angle_start.is_none() {
            bail!("Torque angle start is not set");
        }
        task.torque_angle_start.as_ref().unwrap().parse::<i32>()?
    };
    let torque_upper_tol = {
        if task.torque_deviation_up.is_none() {
            bail!("Torque upper tolerance is not set");
        }
        task.torque_deviation_up.as_ref().unwrap().parse::<i32>()?
    };
    let torque_lower_tol = {
        if task.torque_deviation_down.is_none() {
            bail!("Torque lower tolerance is not set");
        }
        task.torque_deviation_down
            .as_ref()
            .unwrap()
            .parse::<i32>()?
    };
    let angle = {
        if task.angle.is_none() {
            bail!("Angle is not set");
        }
        task.angle.as_ref().unwrap().parse::<i16>()?
    };
    let angle_upper_tol = {
        if task.angle_deviation_up.is_none() {
            bail!("Angle upper tolerance is not set");
        }
        task.angle_deviation_up.as_ref().unwrap().parse::<i16>()?
    };
    let angle_lower_tol = {
        if task.angle_deviation_down.is_none() {
            bail!("Angle lower tolerance is not set");
        }
        task.angle_deviation_down.as_ref().unwrap().parse::<i16>()?
    };
    let task_repeat_times = {
        if task.repeat_count.is_none() {
            bail!("Task repeat times is not set");
        }
        task.repeat_count.as_ref().unwrap().parse::<u16>()?
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
            fdt: 0,
            fda: 0,
            task_repeat_times,
            task_id,
            flag: task_flag,
        }),
    };
    let bytes: Vec<u8> = task_packet.try_into().unwrap();
    debug!("{:02X?}", bytes);
    port.write_all(&sm7bits::encode(&bytes, SM7BitControlBits::WRC))?;
    Ok(())
}

type PendingTask = Vec<(TaskInfo, Vec<(bool, TaskRequestMsg)>)>;

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
    let mut visited: HashSet<u32> = HashSet::new();
    let mut mac_to_tasks: HashMap<u32, PendingTask> = HashMap::new();

    while !exit_required.load(Ordering::Acquire) {
        let readed = reader(exit_required.clone(), &mut port);

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
                    mac_to_tasks.entry(wrc.mac).and_modify(|v| {
                        if status_report.status != 0 {
                            if let Some((task_info, _)) = v.drain(..1).next() {
                                tx.send(ResponseAction::TaskStatus(task_info)).unwrap();
                            };
                        } else {
                            let mut seqid = match last_seqid.get(&wrc.mac) {
                                Some(&s) => s,
                                None => {
                                    error!("last seqid not found");
                                    return;
                                }
                            };
                            for (task_info, task_requests) in v.iter_mut() {
                                for (is_sended, task_request) in task_requests.iter_mut() {
                                    if !*is_sended {
                                        if let Err(e) =
                                            send_task(task_request, seqid + 1, wrc.mac, &mut port)
                                        {
                                            error!("send task failed: {}", e);
                                            tx.send(ResponseAction::TaskStatus(task_info.clone()))
                                                .unwrap();
                                            task_info.status = true;
                                        }
                                        *is_sended = true;
                                        seqid += 1;
                                        break;
                                    }
                                }
                                if seqid == last_seqid[&wrc.mac] {
                                    task_info.status = true;
                                    tx.send(ResponseAction::TaskStatus(task_info.clone()))
                                        .unwrap();
                                } else {
                                    break;
                                }
                            }
                        }
                    });
                }
                WRCPayload::InlineJointData(_) => {
                    debug!("unimplemented!(\"InlineJointData\")");
                }
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
            Ok(RequiredAction::SendTask((msg_id, target))) => {
                let mut task_info = TaskInfo {
                    msg_id,
                    wrench_serial: 0,
                    status: false,
                };
                if target.is_empty() {
                    error!("empty task");
                    continue;
                }
                if target[0].wrench_serial.is_none() {
                    error!("serial number should not be None");
                    continue;
                }
                let wrench_serial =
                    match u128::from_str_radix(target[0].wrench_serial.as_ref().unwrap(), 16) {
                        Ok(s) => s,
                        Err(_) => {
                            error!("invalid serial number");
                            continue;
                        }
                    };
                task_info.wrench_serial = wrench_serial;
                let mac = match serial_to_mac.get(&wrench_serial) {
                    Some(&m) => m,
                    None => {
                        error!("unknown serial number");
                        continue;
                    }
                };
                mac_to_tasks
                    .entry(mac)
                    .and_modify(|v| {
                        v.push((
                            task_info.clone(),
                            target.clone().into_iter().map(|t| (false, t)).collect(),
                        ))
                    })
                    .or_insert(vec![(
                        task_info,
                        target.into_iter().map(|t| (false, t)).collect(),
                    )]);
            }
            Err(_) => {}
        }

        for (mac, task) in mac_to_tasks.iter_mut() {
            let seqid = match last_seqid.get(mac) {
                Some(&s) => s,
                None => {
                    error!("last seqid not found");
                    continue;
                }
            };
            for (task_info, task) in task.iter_mut() {
                if task.iter().all(|(s, _)| *s) {
                    continue;
                }
                if let Some((status, task)) = task.iter_mut().next() {
                    if *status {
                        break;
                    }
                    if let Err(e) = send_task(task, seqid + 1, *mac, &mut port) {
                        error!("send task failed: {}", e);
                        tx.send(ResponseAction::TaskStatus(task_info.clone()))?;
                        task_info.status = true;
                    }
                    *status = true;
                }
                break;
            }
            task.retain(|t| !t.0.status);
        }
    }

    Ok(())
}
