use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkUsage {
    pub network_received: u64,
    pub network_transmitted: u64,
    /* 아래의 옵션은 Linux를 위한 수집용도 필드 */
    pub loop_back_received: u64,
    pub loop_back_transmitted: u64,
    pub ethernet_received: u64,
    pub ethernet_transmitted: u64,
}
