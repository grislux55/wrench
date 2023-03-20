use crate::redis::message::TaskRequestMsg;

#[derive(Debug, Clone, Default)]
pub struct ConnectInfo {
    pub msg_id: String,
    pub wrench_serial: u128,
    pub status: bool,
}

#[derive(Debug, Clone, Default)]
pub struct WrenchInfo {
    pub msg_id: String,
    pub connect_id: String,
    pub wrench_serial: u128,
}

#[derive(Debug, Clone, Default)]
pub struct TaskInfo {
    pub msg_id: String,
    pub wrench_serial: u128,
    pub status: bool,
}

#[derive(Debug, Clone)]
pub enum RequiredAction {
    BindWrench(WrenchInfo),
    CheckConnect(ConnectInfo),
    SendTask((String, Vec<TaskRequestMsg>)),
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    BindResponse(WrenchInfo),
    ConnectStatus(ConnectInfo),
    TaskStatus(TaskInfo),
}
