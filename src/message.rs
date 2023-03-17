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

#[derive(Debug, Clone)]
pub enum RequiredAction {
    BindWrench(WrenchInfo),
    CheckConnect(ConnectInfo),
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    BindStatus(WrenchInfo),
    ConnectStatus(ConnectInfo),
}
