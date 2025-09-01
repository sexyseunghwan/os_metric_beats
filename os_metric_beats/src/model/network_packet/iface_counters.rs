use crate::common::*;

#[derive(Debug, Serialize, Deserialize, Default, Clone, new)]
pub struct IfaceCounters {
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}
