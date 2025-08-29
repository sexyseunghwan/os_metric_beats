use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct LinuxConfig {
    pub network_tx_rx_list: Vec<String>,
}
