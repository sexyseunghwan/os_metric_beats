use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct NetworkPacketInfo {
    pub recv_dropped_packets: i64,
    pub send_dropped_packets: i64,
    pub recv_errors_packet: i64,
    pub send_errors_packet: i64,
}
