use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindRequestMsg {
    pub station_ip: String,
    pub product_serial_no: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindRequest {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: BindRequestMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindResponseMsg {
    pub product_serial_no: String,
    pub wrench_serial: String,
    pub msg_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BindResponse {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: BindResponseMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequestMsg {
    pub station_ip: String,
    pub wrench_serial: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: ConnectRequestMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponseMsg {
    pub wrench_serial: String,
    pub status: String,
    pub desc: String,
    pub msg_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: ConnectResponseMsg,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequestMsg {
    pub station_ip: String,
    pub task_id: String,
    pub task_detail_id: String,
    pub task_desc: Option<String>,
    pub wrench_serial: String,
    pub wrench_serial_desc: Option<String>,
    pub user_id: Option<String>,
    pub user_desc: Option<String>,
    pub control_mode: String,
    pub work_mode: String,
    pub bolt_num: String,
    pub repeat_count: String,
    pub target: String,
    pub monitor: String,
    pub torque: String,
    pub torque_deviation_up: String,
    pub torque_deviation_down: String,
    pub torque_angle_start: String,
    pub angle: String,
    pub angle_deviation_up: String,
    pub angle_deviation_down: String,
    pub unit: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequest {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: Vec<TaskRequestMsg>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponseMsg {
    pub wrench_serial: String,
    pub status: String,
    pub desc: String,
    pub msg_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: TaskResponseMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusMsg {
    pub msg_id: String,
    pub task_id: String,
    pub task_detail_id: String,
    pub wrench_serial: String,
    pub torque: String,
    pub angle: String,
    pub status: String,
    pub consume_time: String,
    pub desc: String,
    pub start_date: String,
    pub end_date: String,
    pub work_time: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: TaskStatusMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskCancelMsg {
    pub task_id: String,
    pub wrench_serial: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskCancel {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: TaskCancelMsg,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MiscInfoMsg {
    pub wrench_serial: String,
    pub title: Option<String>,
    pub code: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub level: Option<String>,
    pub consume_time: Option<String>,
    pub use_time: Option<String>,
    pub storage_num: Option<String>,
    pub voltage: Option<String>,
    pub status: Option<String>,
    pub desc: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MiscInfo {
    pub msg_id: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: MiscInfoMsg,
}
