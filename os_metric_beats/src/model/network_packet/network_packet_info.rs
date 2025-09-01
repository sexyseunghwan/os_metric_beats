use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkPacketInfo {
    pub recv_dropped_packets: u64,
    pub send_dropped_packets: u64,
    pub recv_errors_packet: u64,
    pub send_errors_packet: u64,
}
