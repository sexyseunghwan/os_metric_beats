use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct SystemConfig {
    pub os_server_ip: String,
}
