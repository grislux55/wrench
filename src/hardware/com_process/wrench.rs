use std::{
    collections::{HashSet, VecDeque},
    matches,
    sync::mpsc,
    time::Instant,
};

use anyhow::bail;
use chrono::{DateTime, Local};
use tracing::{debug, error, info};

use crate::{
    hardware::message::wrc::{
        WRCPacket, WRCPacketFlag, WRCPayload, WRCPayloadGetJointData, WRCPayloadInlineJointData,
        WRCPayloadInlineJointDataFlag, WRCPayloadSetJoint, WRCPayloadSetJointFlag,
    },
    message::{BasicInfo, ConnectInfo, FinishedInfo, RequiredAction, ResponseAction, TaskInfo},
    redis::message::TaskRequestMsg,
};

use super::message::query_energy;

#[derive(Debug, Clone)]
pub struct JointTask {
    pub torque: i32,
    pub torque_angle_start: i32,
    pub torque_upper_tol: i32,
    pub torque_lower_tol: i32,
    pub angle: i16,
    pub angle_upper_tol: i16,
    pub angle_lower_tol: i16,
    pub task_repeat_times: u16,
    pub bolt_num: u32,
    pub control_mode: u8,
    pub work_mode: u8,
    pub unit: u8,
}

#[derive(Debug, Clone)]
pub struct JointData {
    pub joint_id: i32,
    pub unix_time: u32,
    pub flag: WRCPayloadInlineJointDataFlag,
    pub torque: i32,
    pub angle: i16,
}

#[derive(Debug, Clone)]
pub struct WrenchTask {
    pub wrench_task_id: u16,
    pub redis_task_id: String,
    pub redis_task_detail_id: String,
    pub msg_id: String,
    pub last_report: DateTime<Local>,
    pub joints_task: JointTask,
    pub joints_recv: Vec<JointData>,
}

#[derive(Debug, Clone)]
pub enum WrenchStatus {
    Connected,
    Working,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct WrenchContext {
    pub mac: u32,
    pub serial: u128,
    pub connect_id: String,
    pub voltage: Option<u16>,
    pub online_time: u64,
    pub last_recv: Instant,
    pub last_send: Instant,
    pub last_send_id: u16,
    pub last_report: Instant,
    pub total_joints: u16,
    pub status: WrenchStatus,
    pub current_task: Option<WrenchTask>,
    pub pending_task: VecDeque<WrenchTask>,
    pub finished_task: Vec<WrenchTask>,
}

impl WrenchContext {
    pub fn new(mac: u32, serial: u128) -> Self {
        let now = Instant::now();

        Self {
            mac,
            serial,
            connect_id: "".to_string(),
            voltage: None,
            online_time: 0,
            last_recv: now,
            last_send: now,
            last_send_id: 0,
            last_report: now,
            total_joints: 0,
            status: WrenchStatus::Connected,
            current_task: None,
            pending_task: VecDeque::new(),
            finished_task: Vec::new(),
        }
    }

    pub fn com_update(&mut self, packet: &WRCPacket, redis_sender: &mpsc::Sender<ResponseAction>) {
        self.online_time = self
            .online_time
            .saturating_add(self.last_recv.elapsed().as_secs());
        self.last_recv = Instant::now();

        // debug!("扳手 {:X} 收到数据包 {:X?}", self.serial, packet);
        match &packet.payload {
            WRCPayload::InfoEnergy(info_energy) => {
                self.voltage = Some(info_energy.battery_voltage_mv);
            }
            WRCPayload::InlineJointData(inline_joint_data) => {
                debug!(
                    "收到扳手 {:X} 的joint 数据 {:?}",
                    self.serial, inline_joint_data
                );
                if let Err(e) = self.process_inline_joint_data(inline_joint_data, redis_sender) {
                    error!("处理来自扳手的 joint 数据失败: {:?}", e);
                }
            }
            _ => {}
        }
    }

    fn process_inline_joint_data(
        &mut self,
        inline_joint_data: &[WRCPayloadInlineJointData],
        tx: &mpsc::Sender<ResponseAction>,
    ) -> Result<(), anyhow::Error> {
        if matches!(self.status, WrenchStatus::Working) {
            let wrench_task = self.current_task.as_mut().unwrap();

            let mut inline_joint_data = inline_joint_data.to_vec();
            inline_joint_data.sort_by_key(|x| x.joint_id);
            let mut joints_set = wrench_task
                .joints_recv
                .iter()
                .map(|x| x.joint_id)
                .collect::<HashSet<_>>();

            let scale_down = |mut int: i32, mut scale: i32| -> String {
                let mut frac = 0;
                let mut level = 0;

                while scale > 0 {
                    frac += i32::pow(10, level) * (int % 10);
                    int /= 10;
                    scale -= 1;
                    level += 1;
                }

                format!("{}.{}", int, frac.abs())
            };

            for recv in inline_joint_data.into_iter() {
                if recv.task_id != wrench_task.wrench_task_id {
                    debug!("不是属于该任务的task_id: {}", recv.task_id);

                    let mut joint_has_recvd = false;
                    for finished in self.finished_task.iter() {
                        if finished
                            .joints_recv
                            .iter()
                            .any(|x| x.joint_id == recv.joint_id as i32)
                        {
                            debug!("重复的joint_id: {}", recv.joint_id);
                            joint_has_recvd = true;
                            break;
                        }
                    }

                    if !joint_has_recvd {
                        debug!("不属于任务的task_id: {}", recv.task_id);
                        self.total_joints += 1;
                    }
                    continue;
                }

                if joints_set.contains(&(recv.joint_id as i32)) {
                    debug!("重复的joint_id: {}", recv.joint_id);
                    continue;
                }

                let tmp = JointData {
                    joint_id: recv.joint_id as i32,
                    unix_time: recv.unix_time,
                    flag: recv.flag.clone(),
                    torque: recv.torque,
                    angle: recv.angle,
                };

                let param = AssertOkParam {
                    torque: wrench_task.joints_task.torque,
                    torque_lower_tol: wrench_task.joints_task.torque_lower_tol,
                    torque_upper_tol: wrench_task.joints_task.torque_upper_tol,
                    angle: wrench_task.joints_task.angle,
                    angle_lower_tol: wrench_task.joints_task.angle_lower_tol,
                    angle_upper_tol: wrench_task.joints_task.angle_upper_tol,
                    control_mode: wrench_task.joints_task.control_mode,
                };

                tx.send(ResponseAction::TaskFinished(FinishedInfo {
                    msg_id: wrench_task.msg_id.clone(),
                    wrench_serial: self.serial,
                    task_id: wrench_task.redis_task_id.clone(),
                    task_detail_id: wrench_task.redis_task_detail_id.clone(),
                    task_sub_id: wrench_task
                        .joints_recv
                        .iter()
                        .filter(|x| assert_ok(&param, x))
                        .count()
                        .to_string(),
                    torque: scale_down(recv.torque, 3),
                    angle: scale_down(recv.angle as i32, 1),
                    status: assert_ok(&param, &tmp),
                    start_date: wrench_task.last_report,
                    end_date: chrono::Local::now(),
                }))?;

                wrench_task.last_report = chrono::Local::now();
                wrench_task.joints_recv.push(tmp);
                joints_set.insert(recv.joint_id as i32);
                self.total_joints += 1;

                debug!(
                    "扳手 {:X} 收集到任务数据: {:?}, 状态为: {:?}",
                    self.serial, recv, self.status
                );
            }
        }

        Ok(())
    }

    pub fn redis_update(
        &mut self,
        recv: RequiredAction,
        com_sender: &mpsc::Sender<WRCPacket>,
        redis_sender: &mpsc::Sender<ResponseAction>,
    ) {
        match recv {
            RequiredAction::CheckConnect(mut connect_info) => {
                connect_info.status = !matches!(self.status, WrenchStatus::Disconnected);
                if let Err(e) = redis_sender.send(ResponseAction::ConnectStatus(connect_info)) {
                    error!("扳手 {:X} 发送连接状态失败: {:?}", self.serial, e);
                }
            }
            RequiredAction::SendTask((msg_id, tasks)) => {
                if let Err(e) = self.append_task(msg_id, tasks, com_sender, redis_sender) {
                    error!("扳手 {:X} 发送任务失败: {:?}", self.serial, e);
                }
            }
            RequiredAction::TaskCancel((_, task_id)) => {
                if let Some(wrench_task) = &self.current_task {
                    if wrench_task.redis_task_id == task_id {
                        self.clear_task(com_sender);
                        self.current_task.take();
                        self.status = WrenchStatus::Connected;
                    }
                }
                self.retain_task(task_id);
                debug!("扳手 {:X} 取消任务", self.serial);
                debug!("扳手 {:X} 当前任务: {:?}", self.serial, self.current_task);
                debug!("扳手 {:X} 任务列表: {:?}", self.serial, self.pending_task);
            }
            _ => {}
        }
    }

    fn retain_task(&mut self, task_id: String) {
        self.pending_task.retain(|x| x.redis_task_id != task_id);
    }

    fn append_task(
        &mut self,
        msg_id: String,
        tasks: Vec<TaskRequestMsg>,
        com_sender: &mpsc::Sender<WRCPacket>,
        redis_sender: &mpsc::Sender<ResponseAction>,
    ) -> Result<(), anyhow::Error> {
        let mut task_info = TaskInfo {
            msg_id: msg_id.clone(),
            wrench_serial: self.serial,
            status: false,
        };

        if tasks.is_empty() {
            redis_sender.send(ResponseAction::TaskStatus(task_info))?;
            bail!("空的任务列表");
        }

        if !matches!(self.status, WrenchStatus::Working) && self.finished_task.is_empty() {
            self.clear_task(com_sender);
        }

        let mut last_task_id = 0;
        last_task_id = last_task_id.max(
            self.finished_task
                .last()
                .map(|x| x.wrench_task_id)
                .unwrap_or(0),
        );
        last_task_id = last_task_id.max(if let Some(x) = &self.current_task {
            x.wrench_task_id
        } else {
            0
        });
        last_task_id = last_task_id.max(
            self.pending_task
                .back()
                .map(|x| x.wrench_task_id)
                .unwrap_or(0),
        );

        let origin_tasks_len = tasks.len();
        let mut need_push = Vec::new();

        for task in tasks {
            let torque = match scale_up(&task.torque, 3) {
                Ok(x) => x,
                Err(_) => continue,
            };
            let torque_angle_start = match scale_up(&task.torque_angle_start, 1) {
                Ok(x) => x,
                Err(_) => continue,
            };
            let torque_upper_tol = match scale_up(&task.torque_deviation_up, 3) {
                Ok(x) => x,
                Err(_) => continue,
            };
            let torque_lower_tol = match scale_up(&task.torque_deviation_down, 3) {
                Ok(x) => x,
                Err(_) => continue,
            };
            let angle = match scale_up(&task.angle, 1) {
                Ok(x) => x as i16,
                Err(_) => continue,
            };
            let angle_upper_tol = match scale_up(&task.angle_deviation_up, 1) {
                Ok(x) => x as i16,
                Err(_) => continue,
            };
            let angle_lower_tol = match scale_up(&task.angle_deviation_down, 1) {
                Ok(x) => x as i16,
                Err(_) => continue,
            };
            let task_repeat_times = match task.repeat_count.parse::<u16>() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let control_mode = match task.control_mode.parse::<u8>() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let work_mode = match task.work_mode.parse::<u8>() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let unit = match task.unit.parse::<u8>() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let bolt_num = match task.bolt_num.parse::<u32>() {
                Ok(x) => x,
                Err(_) => continue,
            };

            last_task_id += 1;
            need_push.push(WrenchTask {
                wrench_task_id: last_task_id,
                redis_task_id: task.task_id,
                redis_task_detail_id: task.task_detail_id,
                msg_id: msg_id.clone(),
                last_report: chrono::Local::now(),
                joints_task: JointTask {
                    torque,
                    torque_angle_start,
                    torque_upper_tol,
                    torque_lower_tol,
                    angle,
                    angle_upper_tol,
                    angle_lower_tol,
                    task_repeat_times,
                    control_mode,
                    work_mode,
                    unit,
                    bolt_num,
                },
                joints_recv: Vec::new(),
            });
        }

        if need_push.len() == origin_tasks_len {
            task_info.status = true;
            self.pending_task.extend(need_push.into_iter());
        }

        redis_sender.send(ResponseAction::TaskStatus(task_info))?;

        Ok(())
    }

    fn clear_task(&mut self, com_sender: &mpsc::Sender<WRCPacket>) {
        let mut flag = WRCPacketFlag(0);
        flag.set_direction(true);
        flag.set_type(10);
        let clear_packet = WRCPacket {
            sequence_id: 0,
            mac: self.mac,
            flag,
            payload_len: 0u8,
            payload: WRCPayload::ClearJointData,
        };
        self.last_send_id = 0;
        self.total_joints = 0;
        if let (WrenchStatus::Working, Some(current)) = (&self.status, &mut self.current_task) {
            for joint in current.joints_recv.iter_mut() {
                joint.joint_id = -1;
            }
        }

        debug!("向Mac地址为: {:X?} 的扳手发送清空任务信号", self.mac);
        if let Err(e) = com_sender.send(clear_packet) {
            error!("扳手 {:X} 发送清空任务失败: {:?}", self.serial, e);
        }
    }

    pub fn interval_update(
        &mut self,
        com_sender: &mpsc::Sender<WRCPacket>,
        redis_sender: &mpsc::Sender<ResponseAction>,
    ) {
        if matches!(self.status, WrenchStatus::Disconnected) {
            if self.last_recv.elapsed() < std::time::Duration::from_secs(5) {
                if self.current_task.is_some() {
                    self.status = WrenchStatus::Working;
                } else {
                    self.status = WrenchStatus::Connected;
                }

                if let Err(e) = redis_sender.send(ResponseAction::ConnectStatus(ConnectInfo {
                    msg_id: "0".to_string(),
                    wrench_serial: self.serial,
                    status: true,
                })) {
                    error!("扳手 {:X} 发送连接状态失败: {:?}", self.serial, e);
                }
            }
            return;
        }

        if self.last_recv.elapsed() > std::time::Duration::from_secs(20) {
            self.status = WrenchStatus::Disconnected;
            info!("扳手 {:X} 连接断开", self.serial);
            if let Err(e) = redis_sender.send(ResponseAction::ConnectionTimeout(self.serial)) {
                error!("扳手 {:X} 无法发送状态: {:?}", self.serial, e);
            }
        }

        if self.last_report.elapsed() > std::time::Duration::from_secs(120) {
            self.last_report = Instant::now();
            query_energy(self.mac, com_sender).ok();
            if let Err(e) = redis_sender.send(ResponseAction::BasicStatus(BasicInfo {
                wrench_serial: self.serial,
                voltage: self.voltage.unwrap_or_default() as u32,
                storage: self.total_joints as u32,
                use_time: self.online_time,
            })) {
                error!("扳手 {:X} 无法发送状态: {:?}", self.serial, e);
            }
        }

        if self.last_send.elapsed() > std::time::Duration::from_secs(5) {
            self.last_send = Instant::now();
            let mut wrc_flag = WRCPacketFlag(0);
            wrc_flag.set_direction(true);
            wrc_flag.set_type(9);
            let get_joint_packet = WRCPacket {
                sequence_id: self.last_send_id.saturating_add(1),
                mac: self.mac,
                flag: wrc_flag,
                payload_len: 3u8,
                payload: WRCPayload::GetJointData(WRCPayloadGetJointData {
                    joint_id_start: self.total_joints,
                    joint_count: 1,
                }),
            };
            self.last_send_id = self.last_send_id.saturating_add(1);
            debug!(
                "向 {:X} 扳手发送查询请求, 该扳手的状态为 {:?}",
                self.serial, self.status
            );
            if let Err(e) = com_sender.send(get_joint_packet) {
                error!("扳手 {:X} 无法发送数据 {:?}", self.serial, e);
            }
        }

        if let Some(wrench_task) = &self.current_task {
            let param = AssertOkParam {
                torque: wrench_task.joints_task.torque,
                torque_lower_tol: wrench_task.joints_task.torque_lower_tol,
                torque_upper_tol: wrench_task.joints_task.torque_upper_tol,
                angle: wrench_task.joints_task.angle,
                angle_lower_tol: wrench_task.joints_task.angle_lower_tol,
                angle_upper_tol: wrench_task.joints_task.angle_upper_tol,
                control_mode: wrench_task.joints_task.control_mode,
            };
            let passed_count = wrench_task
                .joints_recv
                .iter()
                .filter(|x| assert_ok(&param, x))
                .count();
            let target_count = wrench_task.joints_task.bolt_num as usize;

            if passed_count == target_count {
                let tmp = self.current_task.take().unwrap();
                self.finished_task.push(tmp);
                self.status = WrenchStatus::Connected;
            }
        }

        if matches!(self.status, WrenchStatus::Connected) {
            if let Some(wrench_task) = self.pending_task.pop_front() {
                self.current_task = Some(wrench_task);
                self.status = WrenchStatus::Working;
                self.send_task(com_sender);
            }
        }
    }

    pub fn mac_reconnect(
        &mut self,
        mac: u32,
        com_sender: &mpsc::Sender<WRCPacket>,
        redis_sender: &mpsc::Sender<ResponseAction>,
    ) {
        self.mac = mac;
        self.last_recv = Instant::now();

        if matches!(self.status, WrenchStatus::Disconnected) {
            if self.current_task.is_some() {
                self.status = WrenchStatus::Working;
                self.clear_task(com_sender);
                self.send_task(com_sender);
            } else {
                self.status = WrenchStatus::Connected;
            }

            if let Err(e) = redis_sender.send(ResponseAction::ConnectStatus(ConnectInfo {
                msg_id: "0".to_string(),
                wrench_serial: self.serial,
                status: true,
            })) {
                error!("扳手 {:X} 发送连接状态失败: {:?}", self.serial, e);
            }
        }
    }

    fn send_task(&mut self, com_sender: &mpsc::Sender<WRCPacket>) {
        if let Some(wrench_task) = &self.current_task {
            let mut task_flag = WRCPayloadSetJointFlag(0);
            task_flag.set_mode(wrench_task.joints_task.control_mode);
            task_flag.set_method(wrench_task.joints_task.work_mode);
            task_flag.set_unit(wrench_task.joints_task.unit);

            let mut wrc_flag = WRCPacketFlag(0);
            wrc_flag.set_direction(true);
            wrc_flag.set_type(7);
            let task_packet = WRCPacket {
                sequence_id: self.last_send_id.saturating_add(1),
                mac: self.mac,
                flag: wrc_flag,
                payload_len: 33u8,
                payload: WRCPayload::SetJoint(WRCPayloadSetJoint {
                    torque_setpoint: wrench_task.joints_task.torque,
                    torque_angle_start: wrench_task.joints_task.torque_angle_start,
                    torque_upper_tol: wrench_task.joints_task.torque_upper_tol,
                    torque_lower_tol: wrench_task.joints_task.torque_lower_tol,
                    angle: wrench_task.joints_task.angle,
                    angle_upper_tol: wrench_task.joints_task.angle_upper_tol,
                    angle_lower_tol: wrench_task.joints_task.angle_lower_tol,
                    fdt: -1,
                    fda: -1,
                    task_repeat_times: wrench_task.joints_task.bolt_num as u16,
                    task_id: wrench_task.wrench_task_id,
                    flag: task_flag,
                }),
            };

            if let Err(e) = com_sender.send(task_packet) {
                error!("扳手 {:X} 无法发送数据 {:?}", self.serial, e);
            }
        }
    }
}

struct AssertOkParam {
    torque: i32,
    torque_lower_tol: i32,
    torque_upper_tol: i32,
    angle: i16,
    angle_lower_tol: i16,
    angle_upper_tol: i16,
    control_mode: u8,
}

fn assert_ok(param: &AssertOkParam, data: &JointData) -> bool {
    let torque_range = param.torque.saturating_sub(param.torque_lower_tol)
        ..=param.torque.saturating_add(param.torque_upper_tol);
    let angle_range = param.angle.saturating_sub(param.angle_lower_tol)
        ..=param.angle.saturating_add(param.angle_upper_tol);
    match param.control_mode {
        0 => torque_range.contains(&data.torque),
        1 => angle_range.contains(&data.angle),
        _ => torque_range.contains(&data.torque) && angle_range.contains(&data.angle),
    }
}

fn scale_up(s: &str, mut scale: i32) -> anyhow::Result<i32> {
    let mut s = s.split('.');
    let mut int_side = match s.next().unwrap_or("0").parse::<i32>() {
        Ok(x) => x,
        Err(e) => return Err(e.into()),
    };
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
