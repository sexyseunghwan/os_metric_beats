use crate::common::*;

use crate::model::network_packet::iface_counters::*;
use crate::model::network_packet::network_packet_info::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PacketState {
    pub ifaces: HashMap<String, IfaceCounters>,
    pub updated_at: String,
}

impl PacketState {
    pub fn new(updated_at: String) -> Self {
        Self {
            ifaces: HashMap::new(),
            updated_at
        }
    }

    pub fn add_iface(&mut self, name: String, network_packet_info: NetworkPacketInfo) {
        let iface_counter: IfaceCounters = IfaceCounters::new(
            network_packet_info.recv_dropped_packets,
            network_packet_info.send_dropped_packets,
            network_packet_info.recv_errors_packet,
            network_packet_info.send_dropped_packets
        );

        self.ifaces.insert(name, iface_counter);
    }

    pub fn get_iface(&self, name: &str) -> Option<&IfaceCounters> {
        self.ifaces.get(name)
    }

}
