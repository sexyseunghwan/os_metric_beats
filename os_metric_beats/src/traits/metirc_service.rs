use crate::common::*;

use crate::model::network_packet::network_packet_info::*;
use crate::model::network::network_socket_info::*;
use crate::model::network::network_usage::*;
use crate::model::memory::os_mem_res::*;

pub trait MetricService {
    fn get_cpu_usage(&mut self) -> f32;
    fn get_cpu_usage_avg_thread(&mut self) -> f32;
    fn get_disk_usage(&mut self) -> f32;
    fn get_memory_usage(&mut self) -> f32;
    fn get_network_usage(&mut self) -> Result<NetworkUsage, anyhow::Error>;
    fn get_process_count(&mut self) -> usize;
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error>;
    fn get_socket_info_parsing(&mut self, socket_vec: &[&str]) -> (u64, u64);
    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error>;
    fn get_process_mem_usage(&mut self) -> Result<OsMemRes, anyhow::Error>;
}
