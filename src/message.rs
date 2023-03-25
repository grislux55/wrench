use std::fmt::Display;

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

impl Display for RequiredAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredAction::BindWrench(_) => write!(f, "RequiredAction::BindWrench"),
            RequiredAction::CheckConnect(_) => write!(f, "RequiredAction::CheckConnect"),
            RequiredAction::SendTask(_) => write!(f, "RequiredAction::SendTask"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    BindResponse(WrenchInfo),
    ConnectStatus(ConnectInfo),
    TaskStatus(TaskInfo),
}

impl Display for ResponseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseAction::BindResponse(_) => write!(f, "ResponseAction::BindResponse"),
            ResponseAction::ConnectStatus(_) => write!(f, "ResponseAction::ConnectStatus"),
            ResponseAction::TaskStatus(_) => write!(f, "ResponseAction::TaskStatus"),
        }
    }
}
