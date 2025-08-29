use crate::common::*;

use crate::model::network::network_packet_info::*;
use crate::model::network::network_socket_info::*;
use crate::model::network::network_usage::*;

pub trait MetricService {
    fn get_cpu_usage(&mut self) -> f32;
    fn get_cpu_usage_avg_thread(&mut self) -> f32;
    fn get_disk_usage(&mut self) -> f64;
    fn get_memory_usage(&mut self) -> f64;
    fn get_network_usage(&mut self) -> Result<NetworkUsage, anyhow::Error>;
    fn get_process_count(&mut self) -> usize;
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error>;
    fn get_socket_info_parsing(&mut self, socket_vec: &Vec<&str>) -> (i64, i64);
    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error>;
}
