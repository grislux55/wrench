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
    pub serial_no: String,
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

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequestMsg {
    pub station_ip: Option<String>,
    pub task_id: Option<String>,
    pub task_detail_id: Option<String>,
    pub task_desc: Option<String>,
    pub wrench_serial: Option<String>,
    pub wrench_serial_desc: Option<String>,
    pub user_id: Option<String>,
    pub user_desc: Option<String>,
    pub control_mode: Option<String>,
    pub work_mode: Option<String>,
    pub bolt_num: Option<String>,
    pub repeat_count: Option<String>,
    pub target: Option<String>,
    pub monitor: Option<String>,
    pub torque: Option<String>,
    pub torque_deviation_up: Option<String>,
    pub torque_deviation_down: Option<String>,
    pub torque_angle_start: Option<String>,
    pub angle: Option<String>,
    pub angle_deviation_up: Option<String>,
    pub angle_deviation_down: Option<String>,
    pub unit: Option<String>,
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
    pub msg_id: Option<String>,
    pub task_id: Option<String>,
    pub task_detail_id: Option<String>,
    pub wrench_serial: Option<String>,
    pub torque: Option<String>,
    pub angle: Option<String>,
    pub status: Option<String>,
    pub consume_time: Option<String>,
    pub desc: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub work_time: Option<String>,
    pub current_time: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub msg_id: String,
    pub msg_type: String,
    pub handler_name: String,
    pub msg_txt: TaskStatusMsg,
}
