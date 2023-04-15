use std::fmt::Display;

use chrono::{DateTime, Local};

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
    TaskCancel((String, String)),
}

impl Display for RequiredAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredAction::BindWrench(_) => write!(f, "RequiredAction::BindWrench"),
            RequiredAction::CheckConnect(_) => write!(f, "RequiredAction::CheckConnect"),
            RequiredAction::SendTask(_) => write!(f, "RequiredAction::SendTask"),
            RequiredAction::TaskCancel(_) => write!(f, "RequiredAction::TaskCancel"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FinishedInfo {
    pub msg_id: String,
    pub wrench_serial: u128,
    pub task_id: String,
    pub task_detail_id: String,
    pub task_sub_id: String,
    pub torque: String,
    pub angle: String,
    pub status: bool,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct BasicInfo {
    pub wrench_serial: u128,
    pub voltage: u32,
    pub storage: u32,
    pub use_time: u64,
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    BindResponse(WrenchInfo),
    ConnectStatus(ConnectInfo),
    TaskStatus(TaskInfo),
    TaskFinished(FinishedInfo),
    ConnectionTimeout(u128),
    BasicStatus(BasicInfo),
}

impl Display for ResponseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseAction::BindResponse(_) => write!(f, "ResponseAction::BindResponse"),
            ResponseAction::ConnectStatus(_) => write!(f, "ResponseAction::ConnectStatus"),
            ResponseAction::TaskStatus(_) => write!(f, "ResponseAction::TaskStatus"),
            ResponseAction::TaskFinished(_) => write!(f, "ResponseAction::TaskFinished"),
            ResponseAction::ConnectionTimeout(_) => write!(f, "ResponseAction::ConnectionTimeout"),
            ResponseAction::BasicStatus(_) => write!(f, "ResponseAction::BasicStatus"),
        }
    }
}
