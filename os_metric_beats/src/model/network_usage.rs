use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkUsage {
    pub network_received: u64,
    pub network_transmitted: u64,
}
