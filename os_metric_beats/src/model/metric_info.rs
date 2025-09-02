use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, Builder)]
#[builder(setter(into), default)]
pub struct MetricInfo {
    pub timestamp: String,
    pub host: String,
    pub system_cpu_usage: f32,
    pub system_disk_usage: f64,
    pub system_memory_usage: f64,
    pub network_received: u64,
    pub network_transmitted: u64,
    pub process_count: usize,
    pub recv_dropped_packets: u64,
    pub send_dropped_packets: u64,
    pub recv_errors_packet: u64,
    pub send_errors_packet: u64,
    pub tcp_connections: i32,
    pub udp_sockets: i32,
    pub tcp_established: i32,
    pub tcp_timewait: i32,
    pub tcp_listen: i32,
    pub tcp_close_wait: i32,
    pub process_use_mem: u64,
    pub process_virtual_mem: u64,
}

impl Default for MetricInfo {
    fn default() -> Self {
        MetricInfo {
            timestamp: String::new(),
            host: String::new(),
            system_cpu_usage: 0.0,
            system_disk_usage: 0.0,
            system_memory_usage: 0.0,
            network_received: 0,
            network_transmitted: 0,
            process_count: 0,
            recv_dropped_packets: 0,
            send_dropped_packets: 0,
            recv_errors_packet: 0,
            send_errors_packet: 0,
            tcp_connections: 0,
            udp_sockets: 0,
            tcp_established: 0,
            tcp_timewait: 0,
            tcp_listen: 0,
            tcp_close_wait: 0,
            process_use_mem: 0,
            process_virtual_mem: 0,
        }
    }
}