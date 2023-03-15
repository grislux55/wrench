#[derive(Debug, Clone)]
pub struct ConnectInfo {
    pub msg_id: String,
    pub wrench_serial: u128,
}

#[derive(Debug, Clone)]
pub enum Action {
    CheckConnect(ConnectInfo),
    ConnectStatus((bool, ConnectInfo)),
}
