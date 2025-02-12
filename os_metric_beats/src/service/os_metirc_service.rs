use crate::common::*;

use crate::model::network_packet_info::*;
use crate::model::network_socket_info::*;
use crate::model::network_usage::*;

pub trait MetricService {
    fn get_cpu_usage(&mut self) -> f32;
    fn get_cpu_usage_avg_thread(&mut self) -> f32;
    fn get_disk_usage(&mut self) -> f64;
    fn get_memory_usage(&mut self) -> f64;
    fn get_network_usage(&mut self) -> NetworkUsage;
    fn get_process_count(&mut self) -> usize;
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error>;
    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error>;
}

#[derive(Debug)]
pub struct MetricServicePub {
    system: System,
}

impl MetricServicePub {
    pub fn new() -> Self {
        let mut system: System = System::new_all();
        system.refresh_all(); /* 시스템 정보 초기화 */
        MetricServicePub { system }
    }
}

impl MetricService for MetricServicePub {
    #[doc = "cpu 의 사용률을 체크. - cpu Max 값 추출"]
    fn get_cpu_usage(&mut self) -> f32 {
        /* 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트) */
        self.system.refresh_cpu();

        let mut max_cpu_val: f32 = 0.0;

        for cpu in self.system.cpus() {
            let thread_cpu_usage: f32 = cpu.cpu_usage();
            max_cpu_val = max_cpu_val.max(thread_cpu_usage);
        }

        max_cpu_val.round() * 100.0 / 100.0
    }

    #[doc = "cpu 의 평균 사용률을 체크. - 스레드별 평균"]
    fn get_cpu_usage_avg_thread(&mut self) -> f32 {
        /* 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트) */
        self.system.refresh_cpu();

        let cpu_usage_sum: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let cpu_thread_cnt: usize = self.system.cpus().len();

        if cpu_thread_cnt == 0 {
            return 0.0;
        }

        let cpu_usage_avg: f32 = cpu_usage_sum / cpu_thread_cnt as f32;
        cpu_usage_avg.round() * 100.0 / 100.0
    }

    #[doc = "disk 사용률을 체크"]
    fn get_disk_usage(&mut self) -> f64 {
        self.system.refresh_disks_list();

        if let Some(disk) = self.system.disks().iter().next() {
            let total_space: f64 = disk.total_space() as f64;
            let available_space: f64 = disk.available_space() as f64;
            let used_space: f64 = total_space - available_space;

            let usage_percentage: f64 = (used_space / total_space) * 100.0;
            return (usage_percentage * 100.0).round() / 100.0;
        }

        0.0
    }

    #[doc = "memory 사용률을 체크"]
    fn get_memory_usage(&mut self) -> f64 {
        self.system.refresh_memory();

        let total_memory: f64 = self.system.total_memory() as f64;
        let used_memory: f64 = self.system.used_memory() as f64;

        /* 사용된 메모리 비율 계산 */
        let usage_percentage: f64 = (used_memory / total_memory) * 100.0;

        /* 소수점 둘째 자리에서 반올림 */
        (usage_percentage * 100.0).round() / 100.0
    }

    #[doc = "Network 사용량 체크"]
    fn get_network_usage(&mut self) -> NetworkUsage {
        self.system.refresh_networks_list();

        let networks: &sysinfo::Networks = self.system.networks();
        let mut network_received: u64 = 0;
        let mut network_transmitted: u64 = 0;

        for (_interface_name, network) in networks.iter() {
            network_received += network.received();
            network_transmitted += network.transmitted();
        }

        NetworkUsage::new(network_received, network_transmitted)
    }

    #[doc = "현재 동작중인 프로세스의 개수"]
    fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();

        let process_count: usize = self.system.processes().len();

        process_count
    }

    #[doc = "네트워크 패킷정보를 반환해주는 함수"]
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error> {
        let output: Vec<u8> = std::process::Command::new("netstat")
            .args(["-e"])
            .output()?
            .stdout;

        let output_str: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output);

        let mut dropped_packets: i64 = 0;
        let mut errors_packet: i64 = 0;

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if line.contains("버림") {
                dropped_packets = match parts.last() {
                    Some(parts) => parts.parse::<i64>().unwrap_or(0),
                    None => {
                        error!("[Error][get_network_packet_infos()] The value 'droped_packets' does not exist.");
                        0
                    }
                }
            }

            if line.contains("오류") {
                errors_packet = match parts.last() {
                    Some(parts) => parts.parse::<i64>().unwrap_or(0),
                    None => {
                        error!("[Error][get_network_packet_infos()] The value 'errors' does not exist.");
                        0
                    }
                }
            }
        }

        let network_packet_info: NetworkPacketInfo =
            NetworkPacketInfo::new(dropped_packets, errors_packet);

        Ok(network_packet_info)
    }

    #[doc = "소켓정보를 반환해주는 함수"]
    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error> {
        let af_flags: AddressFamilyFlags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags: ProtocolFlags = ProtocolFlags::TCP | ProtocolFlags::UDP;

        let sockets: Vec<SocketInfo> = get_sockets_info(af_flags, proto_flags)?;

        let mut tcp_connections: i32 = 0;
        let mut udp_sockets: i32 = 0;
        let mut tcp_established: i32 = 0;
        let mut tcp_timewait: i32 = 0;
        let mut tcp_listen: i32 = 0;
        let mut tcp_close_wait: i32 = 0;

        for socket in sockets {
            match socket.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_info) => {
                    tcp_connections += 1;
                    if tcp_info.state == TcpState::Listen {
                        tcp_listen += 1;
                    } else if tcp_info.state == TcpState::Established {
                        tcp_established += 1;
                    } else if tcp_info.state == TcpState::TimeWait {
                        tcp_timewait += 1;
                    } else if tcp_info.state == TcpState::CloseWait {
                        tcp_close_wait += 1;
                    }
                }
                ProtocolSocketInfo::Udp(_) => {
                    udp_sockets += 1;
                }
            }
        }

        let network_socket_info: NetworkSocketInfo = NetworkSocketInfo::new(
            tcp_connections,
            udp_sockets,
            tcp_established,
            tcp_timewait,
            tcp_listen,
            tcp_close_wait,
        );

        Ok(network_socket_info)
    }
}
