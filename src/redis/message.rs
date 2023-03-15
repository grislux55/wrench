use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResquestMsg {
    pub station_ip: Option<String>,
    pub task_id: Option<String>,
    pub wrench_name: Option<String>,
    pub wrench_serial: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResqust {
    pub msg_id: String,
    pub msg_type: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: ConnectResquestMsg,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponseMsg {
    pub wrench_name: Option<String>,
    pub wrench_serial: Option<String>,
    pub status: Option<String>,
    pub desc: Option<String>,
    pub current_time: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub msg_type: String,
    pub msg_id: String,
    pub handler_name: String,
    pub msg_txt: ConnectResponseMsg,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskRequest {
    pub msg_id: String,
    pub msg_type: String,
    pub handler_name: String,
    pub current_time: String,
    pub msg_txt: TaskRequestMsg,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponseMsg {
    pub wrench_serial: Option<String>,
    pub current_time: Option<String>,
    pub status: Option<String>,
    pub desc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskResponse {
    pub msg_id: String,
    pub msg_type: String,
    pub handler_name: String,
    pub msg_txt: TaskResponseMsg,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatus {
    pub msg_id: String,
    pub msg_type: String,
    pub handler_name: String,
    pub msg_txt: TaskStatusMsg,
}
