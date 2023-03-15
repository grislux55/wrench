use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConnectResquestMsg {
    station_ip: Option<String>,
    task_id: Option<String>,
    wrench_name: Option<String>,
    wrench_serial: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResqust {
    msg_id: String,
    msg_type: String,
    handler_name: String,
    current_time: String,
    msg_txt: ConnectResquestMsg,
}
