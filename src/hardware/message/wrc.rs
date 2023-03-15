#[derive(Debug)]
pub enum WRCJointDataMode {
    Torque = 0,
    Angle,
    TorqueAngle,
    AngleTorque,
}

#[derive(Debug)]
pub enum WRCJointDataMethod {
    Click = 0,
    Peak,
    Track,
}

#[derive(Debug)]
pub enum WRCJointDataUnit {
    Nm = 0,
    Inlb,
    Ftlb,
}

#[derive(Debug)]
pub enum WRCPacketDirection {
    FromClient = 0,
    FromServer = 1,
}

#[derive(Debug)]
pub enum WRCPacketType {
    Unknown = 0,
    InfoGeneric,
    InfoSerial,
    InfoTiming,
    InfoEnergy,
    InfoNetwork,
    GetInfo,
    SetJoint,
    SetWrenchTime,
    GetJointData,
    ClearJointData,
    GetStatusReport,
    Beep,
    JointData,
    StatusReport,
    InlineJointData,
}

#[derive(Debug)]
pub enum WRCStatus {
    None = 0,
    Success,
    Failed,
    JointsDeleted,
    GetJointSuccess,
    GetJointRangeError,
}

#[derive(Debug)]
pub struct WRCPayloadInfoSerial {
    pub serial: [u8; 16],
}

#[derive(Debug)]
pub struct WRCPayloadInfoGeneric {
    pub joint_count: u16,
    pub last_server_packet_seqid: u16,
}

#[derive(Debug)]
pub struct WRCPayloadInfoTiming {
    pub cpu_ticks: u32,
    pub wrench_time: u32,
}

#[derive(Debug)]
pub struct WRCPayloadInfoNetworkPackets {
    pub collisions: u16,
    pub crc_errors: u16,
    pub tx_count: u16,
    pub rx_wanted_count: u16,
    pub rx_unwanted_count: u16,
}

#[derive(Debug)]
pub struct WRCPayloadInfoNetworkRF {
    pub rx_rssi: i8,
    pub rx_snr: i8,
    pub rx_rscp: i8,
}

#[derive(Debug)]
pub struct WRCPayloadInfoNetwork {
    pub packets: WRCPayloadInfoNetworkPackets,
    pub rf: WRCPayloadInfoNetworkRF,
}

bitfield::bitfield! {
    pub struct WRCPayloadInfoEnergyFlag(u8);
    impl Debug;
    u8;
    pub is_charging, set_charging: 0;
    pub is_hibernated, set_hibernated: 1;
    pub is_power_connected, set_power_connected: 2;
    pub is_f3, set_f3: 3;
    pub is_f4, set_f4: 4;
    pub is_f5, set_f5: 5;
    pub is_f6, set_f6: 6;
    pub is_f7, set_f7: 7;
}

#[derive(Debug)]
pub struct WRCPayloadInfoEnergy {
    pub flag: WRCPayloadInfoEnergyFlag,
    pub battery_voltage_mv: u16,
}

bitfield::bitfield! {
    pub struct WRCPayloadInlineJointDataFlag(u8);
    impl Debug;
    u8;
    pub is_valid, set_valid: 0;
    pub is_ok, set_ok: 1;
    pub get_mode, set_mode: 3, 2;
    pub get_method, set_method: 5, 4;
    pub get_unit, set_unit: 7, 6;
}

#[derive(Debug)]
pub struct WRCPayloadInlineJointData {
    pub joint_id: u16,
    pub task_id: u16,
    pub unix_time: u32,
    pub flag: WRCPayloadInlineJointDataFlag,
    pub torque: i32,
    pub angle: i16,
}

bitfield::bitfield! {
    pub struct WRCPayloadJointDataFlag(u8);
    impl Debug;
    u8;
    pub get_rsvd, set_rsvd: 1, 0;
    pub get_mode, set_mode: 3, 2;
    pub get_method, set_method: 5, 4;
    pub get_unit, set_unit: 7, 6;
}

#[derive(Debug)]
pub struct WRCPayloadSetJoint {
    pub torque_setpoint: i32,
    pub torque_angle_start: i32,
    pub torque_upper_tol: i32,
    pub torque_lower_tol: i32,
    pub angle: i16,
    pub angle_upper_tol: i16,
    pub angle_lower_tol: i16,
    pub fdt: i32,
    pub fda: i16,
    pub task_repeat_times: u16,
    pub task_id: u16,
    pub flag: WRCPayloadJointDataFlag,
}

#[derive(Debug)]
pub struct WRCPayloadSetWrenchTime {
    pub unix_time: u32,
}

#[derive(Debug)]
pub struct WRCPayloadGetJointData {
    pub joint_id_start: u16,
    pub joint_count: u8,
}

#[derive(Debug)]
pub struct WRCPayloadStatusReport {
    pub target_seqid: u16,
    pub status: u16,
}

bitfield::bitfield! {
    pub struct WRCPayloadGetInfoFlag(u8);
    impl Debug;
    u8;
    pub is_serial, set_serial: 0;
    pub is_generic, set_generic: 1;
    pub is_energy, set_energy: 2;
    pub is_timing, set_timing: 3;
    pub is_network, set_network: 4;
    pub is_f5, set_f5: 5;
    pub is_f6, set_f6: 6;
    pub is_f7, set_f7: 7;
}

#[derive(Debug)]
pub struct WRCPayloadGetInfo {
    pub flag: WRCPayloadGetInfoFlag,
}

#[derive(Debug)]
pub enum WRCPayload {
    InfoGeneric(WRCPayloadInfoGeneric),
    InfoSerial(WRCPayloadInfoSerial),
    InfoTiming(WRCPayloadInfoTiming),
    InfoEnergy(WRCPayloadInfoEnergy),
    InfoNetwork(WRCPayloadInfoNetwork),
    GetInfo(WRCPayloadGetInfo),
    SetJoint(WRCPayloadSetJoint),
    SetWrenchTime(WRCPayloadSetWrenchTime),
    GetJointData(WRCPayloadGetJointData),
    ClearJointData,
    GetStatusReport,
    Beep,
    JointData,
    StatusReport(WRCPayloadStatusReport),
    InlineJointData(WRCPayloadInlineJointData),
}

bitfield::bitfield! {
    pub struct WRCPacketFlag(u8);
    impl Debug;
    u8;
    pub get_direction, set_direction: 0;
    pub is_variable_len, set_variable_len: 1;
    pub get_type, set_type: 7, 2;
}

#[derive(Debug)]
pub struct WRCPacket {
    pub sequence_id: u16,
    pub mac: u32,
    pub flag: WRCPacketFlag,
    pub payload_len: u8,
    pub payload: WRCPayload,
}

impl TryFrom<Vec<u8>> for WRCPacket {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 9 {
            return Err("Packet too short");
        }

        let mut sequence_id = [0u8; 2];
        sequence_id.copy_from_slice(&value[0..2]);
        let sequence_id = u16::from_le_bytes(sequence_id);
        let mut mac = [0u8; 4];
        mac.copy_from_slice(&value[2..6]);
        let mac = u32::from_le_bytes(mac);
        let flag = WRCPacketFlag(value[6]);
        let payload_len = value[7];
        let payload = &value[8..];

        if payload.len() != payload_len as usize {
            return Err("Payload length mismatch");
        }

        let payload = match flag.get_type() {
            1 => {
                let mut joint_count = [0u8; 2];
                joint_count.copy_from_slice(&payload[0..2]);
                let joint_count = u16::from_le_bytes(joint_count);
                let mut last_server_packet_seqid = [0u8; 2];
                last_server_packet_seqid.copy_from_slice(&payload[2..4]);
                let last_server_packet_seqid = u16::from_le_bytes(last_server_packet_seqid);
                WRCPayload::InfoGeneric(WRCPayloadInfoGeneric {
                    joint_count,
                    last_server_packet_seqid,
                })
            }
            2 => {
                let mut serial = [0u8; 16];
                serial.copy_from_slice(&payload[0..16]);
                WRCPayload::InfoSerial(WRCPayloadInfoSerial { serial })
            }
            3 => {
                let mut cpu_ticks = [0u8; 4];
                cpu_ticks.copy_from_slice(&payload[0..4]);
                let cpu_ticks = u32::from_le_bytes(cpu_ticks);
                let mut wrench_time = [0u8; 4];
                wrench_time.copy_from_slice(&payload[4..8]);
                let wrench_time = u32::from_le_bytes(wrench_time);
                WRCPayload::InfoTiming(WRCPayloadInfoTiming {
                    cpu_ticks,
                    wrench_time,
                })
            }
            4 => {
                let flag = WRCPayloadInfoEnergyFlag(payload[0]);
                let mut battery_voltage_mv = [0u8; 2];
                battery_voltage_mv.copy_from_slice(&payload[1..3]);
                let battery_voltage_mv = u16::from_le_bytes(battery_voltage_mv);
                WRCPayload::InfoEnergy(WRCPayloadInfoEnergy {
                    flag,
                    battery_voltage_mv,
                })
            }
            5 => {
                let packets = {
                    let mut collisions = [0u8; 2];
                    collisions.copy_from_slice(&payload[0..2]);
                    let collisions = u16::from_le_bytes(collisions);
                    let mut crc_errors = [0u8; 2];
                    crc_errors.copy_from_slice(&payload[2..4]);
                    let crc_errors = u16::from_le_bytes(crc_errors);
                    let mut tx_count = [0u8; 2];
                    tx_count.copy_from_slice(&payload[4..6]);
                    let tx_count = u16::from_le_bytes(tx_count);
                    let mut rx_wanted_count = [0u8; 2];
                    rx_wanted_count.copy_from_slice(&payload[6..8]);
                    let rx_wanted_count = u16::from_le_bytes(rx_wanted_count);
                    let mut rx_unwanted_count = [0u8; 2];
                    rx_unwanted_count.copy_from_slice(&payload[8..10]);
                    let rx_unwanted_count = u16::from_le_bytes(rx_unwanted_count);
                    WRCPayloadInfoNetworkPackets {
                        collisions,
                        crc_errors,
                        tx_count,
                        rx_wanted_count,
                        rx_unwanted_count,
                    }
                };
                let rf = {
                    let rx_rssi = payload[10] as i8;
                    let rx_snr = payload[11] as i8;
                    let rx_rscp = payload[12] as i8;
                    WRCPayloadInfoNetworkRF {
                        rx_rssi,
                        rx_snr,
                        rx_rscp,
                    }
                };
                WRCPayload::InfoNetwork(WRCPayloadInfoNetwork { packets, rf })
            }
            6 => {
                let flag = WRCPayloadGetInfoFlag(payload[0]);
                WRCPayload::GetInfo(WRCPayloadGetInfo { flag })
            }
            7 => {
                let mut torque_setpoint = [0u8; 4];
                torque_setpoint.copy_from_slice(&payload[0..4]);
                let torque_setpoint = i32::from_le_bytes(torque_setpoint);
                let mut torque_angle_start = [0u8; 4];
                torque_angle_start.copy_from_slice(&payload[4..8]);
                let torque_angle_start = i32::from_le_bytes(torque_angle_start);
                let mut torque_upper_tol = [0u8; 4];
                torque_upper_tol.copy_from_slice(&payload[8..12]);
                let torque_upper_tol = i32::from_le_bytes(torque_upper_tol);
                let mut torque_lower_tol = [0u8; 4];
                torque_lower_tol.copy_from_slice(&payload[12..16]);
                let torque_lower_tol = i32::from_le_bytes(torque_lower_tol);
                let mut angle = [0u8; 2];
                angle.copy_from_slice(&payload[16..18]);
                let angle = i16::from_le_bytes(angle);
                let mut angle_upper_tol = [0u8; 2];
                angle_upper_tol.copy_from_slice(&payload[18..20]);
                let angle_upper_tol = i16::from_le_bytes(angle_upper_tol);
                let mut angle_lower_tol = [0u8; 2];
                angle_lower_tol.copy_from_slice(&payload[20..22]);
                let angle_lower_tol = i16::from_le_bytes(angle_lower_tol);
                let mut fdt = [0u8; 4];
                fdt.copy_from_slice(&payload[22..26]);
                let fdt = i32::from_le_bytes(fdt);
                let mut fda = [0u8; 2];
                fda.copy_from_slice(&payload[26..28]);
                let fda = i16::from_le_bytes(fda);
                let mut task_repeat_times = [0u8; 2];
                task_repeat_times.copy_from_slice(&payload[28..30]);
                let task_repeat_times = u16::from_le_bytes(task_repeat_times);
                let mut task_id = [0u8; 2];
                task_id.copy_from_slice(&payload[30..32]);
                let task_id = u16::from_le_bytes(task_id);
                let flag = WRCPayloadJointDataFlag(payload[32]);
                WRCPayload::SetJoint(WRCPayloadSetJoint {
                    torque_setpoint,
                    torque_angle_start,
                    torque_upper_tol,
                    torque_lower_tol,
                    angle,
                    angle_upper_tol,
                    angle_lower_tol,
                    fdt,
                    fda,
                    task_repeat_times,
                    task_id,
                    flag,
                })
            }
            8 => {
                let mut unix_time = [0u8; 4];
                unix_time.copy_from_slice(&payload[0..4]);
                let unix_time = u32::from_le_bytes(unix_time);
                WRCPayload::SetWrenchTime(WRCPayloadSetWrenchTime { unix_time })
            }
            9 => {
                let mut joint_id_start = [0u8; 2];
                joint_id_start.copy_from_slice(&payload[0..2]);
                let joint_id_start = u16::from_le_bytes(joint_id_start);
                let joint_count = payload[2];
                WRCPayload::GetJointData(WRCPayloadGetJointData {
                    joint_id_start,
                    joint_count,
                })
            }
            10 => WRCPayload::ClearJointData,
            11 => WRCPayload::GetStatusReport,
            12 => WRCPayload::Beep,
            13 => WRCPayload::JointData,
            14 => {
                let mut target_seqid = [0u8; 2];
                target_seqid.copy_from_slice(&payload[0..2]);
                let target_seqid = u16::from_le_bytes(target_seqid);
                let mut status = [0u8; 2];
                status.copy_from_slice(&payload[2..4]);
                let status = u16::from_le_bytes(status);
                WRCPayload::StatusReport(WRCPayloadStatusReport {
                    target_seqid,
                    status,
                })
            }
            15 => {
                let mut joint_id = [0u8; 2];
                joint_id.copy_from_slice(&payload[0..2]);
                let joint_id = u16::from_le_bytes(joint_id);
                let mut task_id = [0u8; 2];
                task_id.copy_from_slice(&payload[2..4]);
                let task_id = u16::from_le_bytes(task_id);
                let mut unix_time = [0u8; 4];
                unix_time.copy_from_slice(&payload[4..8]);
                let unix_time = u32::from_le_bytes(unix_time);
                let flag = WRCPayloadInlineJointDataFlag(payload[8]);
                let mut torque = [0u8; 4];
                torque.copy_from_slice(&payload[9..13]);
                let torque = i32::from_le_bytes(torque);
                let mut angle = [0u8; 2];
                angle.copy_from_slice(&payload[13..15]);
                let angle = i16::from_le_bytes(angle);
                WRCPayload::InlineJointData(WRCPayloadInlineJointData {
                    joint_id,
                    task_id,
                    unix_time,
                    flag,
                    torque,
                    angle,
                })
            }
            _ => {
                return Err("Unknown packet type");
            }
        };

        Ok(WRCPacket {
            sequence_id,
            mac,
            flag,
            payload_len,
            payload,
        })
    }
}

impl TryInto<Vec<u8>> for WRCPacket {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let mut result = vec![];
        result.extend_from_slice(&self.sequence_id.to_le_bytes());
        result.extend_from_slice(&self.mac.to_le_bytes());
        result.push(self.flag.0);
        result.extend_from_slice(&self.payload_len.to_le_bytes());

        match self.payload {
            WRCPayload::InfoGeneric(info_generic) => {
                result.extend_from_slice(&info_generic.joint_count.to_le_bytes());
                result.extend_from_slice(&info_generic.last_server_packet_seqid.to_le_bytes());
            }
            WRCPayload::InfoSerial(info_serial) => {
                result.extend_from_slice(&info_serial.serial);
            }
            WRCPayload::InfoTiming(info_timing) => {
                result.extend_from_slice(&info_timing.cpu_ticks.to_le_bytes());
                result.extend_from_slice(&info_timing.wrench_time.to_le_bytes());
            }
            WRCPayload::InfoEnergy(info_energy) => {
                result.push(info_energy.flag.0);
                result.extend_from_slice(&info_energy.battery_voltage_mv.to_le_bytes());
            }
            WRCPayload::InfoNetwork(info_network) => {
                result.extend_from_slice(&info_network.packets.collisions.to_le_bytes());
                result.extend_from_slice(&info_network.packets.crc_errors.to_le_bytes());
                result.extend_from_slice(&info_network.packets.tx_count.to_le_bytes());
                result.extend_from_slice(&info_network.packets.rx_wanted_count.to_le_bytes());
                result.extend_from_slice(&info_network.packets.rx_unwanted_count.to_le_bytes());
                result.extend_from_slice(&info_network.rf.rx_rssi.to_le_bytes());
                result.extend_from_slice(&info_network.rf.rx_snr.to_le_bytes());
                result.extend_from_slice(&info_network.rf.rx_rscp.to_le_bytes());
            }
            WRCPayload::GetInfo(get_info) => {
                result.push(get_info.flag.0);
            }
            WRCPayload::SetJoint(set_joint) => {
                result.extend_from_slice(&set_joint.torque_setpoint.to_le_bytes());
                result.extend_from_slice(&set_joint.torque_angle_start.to_le_bytes());
                result.extend_from_slice(&set_joint.torque_upper_tol.to_le_bytes());
                result.extend_from_slice(&set_joint.torque_lower_tol.to_le_bytes());
                result.extend_from_slice(&set_joint.angle.to_le_bytes());
                result.extend_from_slice(&set_joint.angle_upper_tol.to_le_bytes());
                result.extend_from_slice(&set_joint.angle_lower_tol.to_le_bytes());
                result.extend_from_slice(&set_joint.fdt.to_le_bytes());
                result.extend_from_slice(&set_joint.fda.to_le_bytes());
                result.extend_from_slice(&set_joint.task_repeat_times.to_le_bytes());
                result.extend_from_slice(&set_joint.task_id.to_le_bytes());
                result.push(set_joint.flag.0);
            }
            WRCPayload::SetWrenchTime(set_wrench_time) => {
                result.extend_from_slice(&set_wrench_time.unix_time.to_le_bytes());
            }
            WRCPayload::GetJointData(get_joint_data) => {
                result.extend_from_slice(&get_joint_data.joint_id_start.to_le_bytes());
                result.extend_from_slice(&get_joint_data.joint_count.to_le_bytes());
            }
            WRCPayload::ClearJointData => {}
            WRCPayload::GetStatusReport => {}
            WRCPayload::Beep => {}
            WRCPayload::JointData => {}
            WRCPayload::StatusReport(status_report) => {
                result.extend_from_slice(&status_report.target_seqid.to_le_bytes());
                result.extend_from_slice(&status_report.status.to_le_bytes());
            }
            WRCPayload::InlineJointData(inline_joint_data) => {
                result.extend_from_slice(&inline_joint_data.joint_id.to_le_bytes());
                result.extend_from_slice(&inline_joint_data.task_id.to_le_bytes());
                result.extend_from_slice(&inline_joint_data.unix_time.to_le_bytes());
                result.push(inline_joint_data.flag.0);
                result.extend_from_slice(&inline_joint_data.torque.to_le_bytes());
                result.extend_from_slice(&inline_joint_data.angle.to_le_bytes());
            }
        }

        Ok(result)
    }
}
