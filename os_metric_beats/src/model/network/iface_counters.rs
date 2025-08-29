use crate::common::*;

#[derive(Debug, Serialize, Deserialize, Default, Clone, new)]
pub struct IfaceCounters {
    pub rx: u64,
    pub tx: u64,
}
