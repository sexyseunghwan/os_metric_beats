use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct MetricInfo {
    pub timestamp: String,
    pub host: String,
    pub system_cpu_usage: f32,
    pub system_disk_usage: f64,
    pub system_memory_usage: f64,
    pub network_received: u64,
    pub network_transmitted: u64,
    pub process_count: usize,
    pub dropped_packets: i64,
    pub errors_packet: i64,
    pub tcp_connections: i32,
    pub udp_sockets: i32,
    pub tcp_established: i32,
    pub tcp_timewait: i32,
    pub tcp_listen: i32,
    pub tcp_close_wait: i32,
}