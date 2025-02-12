use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkPacketInfo {
    pub dropped_packets: i64,
    pub errors_packet: i64,
}
