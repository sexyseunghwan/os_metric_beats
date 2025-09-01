use crate::common::*;

use crate::model::network_packet::network_packet_info::*;
use crate::model::network::network_socket_info::*;
use crate::model::network::network_usage::*;
use crate::model::memory::os_mem_res::*;
use crate::traits::metirc_service::*;

#[derive(Debug)]
pub struct WindowsMetricServiceImpl {
    system: System,
}

impl WindowsMetricServiceImpl {
    pub fn new() -> Self {
        let mut system: System = System::new_all();
        system.refresh_all(); /* 시스템 정보 초기화 */
        WindowsMetricServiceImpl { system }
    }
}

impl MetricService for WindowsMetricServiceImpl {
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

        /* D:\ 드라이브를 찾기 */
        if let Some(disk) = self.system.disks().iter().find(|d| {
            d.mount_point()
                .to_str()
                .map_or(false, |path| path.starts_with("D:\\"))
        }) {
            let total_space: f64 = disk.total_space() as f64;
            let available_space: f64 = disk.available_space() as f64;
            let used_space: f64 = total_space - available_space;
            let usage_percentage: f64 = (used_space / total_space) * 100.0;

            /* 소수점 둘째 자리 반올림 */
            return (usage_percentage * 100.0).round() / 100.0;
        }

        // if let Some(disk) = self.system.disks().iter().next() {

        //     println!("{:?}", disk);

        //     let total_space: f64 = disk.total_space() as f64;
        //     let available_space: f64 = disk.available_space() as f64;
        //     let used_space: f64 = total_space - available_space;

        //     let usage_percentage: f64 = (used_space / total_space) * 100.0;
        //     return (usage_percentage * 100.0).round() / 100.0;
        // }

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
    fn get_network_usage(&mut self) -> Result<NetworkUsage, anyhow::Error> {
        self.system.refresh_networks_list();

        let networks: &sysinfo::Networks = self.system.networks();
        let mut network_received: u64 = 0;
        let mut network_transmitted: u64 = 0;

        for (_interface_name, network) in networks.iter() {
            network_received += network.received();
            network_transmitted += network.transmitted();
        }

        Ok(NetworkUsage::new(
            network_received,
            network_transmitted,
            0,
            0,
            0,
            0,
        ))
    }

    #[doc = "현재 동작중인 프로세스의 개수"]
    fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();

        let process_count: usize = self.system.processes().len();

        process_count
    }

    #[doc = "네트워크 패킷정보를 반환해주는 함수 - 파싱 함수"]
    /// # Arguments
    /// * `socket_vec` - 소켓정보가 들어있는 벡터
    ///
    /// # Returns
    /// * (i64, i64)
    fn get_socket_info_parsing(&mut self, socket_vec: &Vec<&str>) -> (u64, u64) {
        let recv_packet: u64 = match socket_vec.get(socket_vec.len() - 2) {
            Some(recv_packet) => recv_packet.parse::<u64>().unwrap_or(0),
            None => {
                error!("[Error][get_socket_info_parsing()] The value 'recv_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        let send_packet: u64 = match socket_vec.last() {
            Some(send_packet) => send_packet.parse::<u64>().unwrap_or(0),
            None => {
                error!("[Error][get_socket_info_parsing()] The value 'send_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        (recv_packet, send_packet)
    }

    #[doc = "네트워크 패킷정보를 반환해주는 함수"]
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error> {
        let output: Vec<u8> = std::process::Command::new("netstat")
            .args(["-e"])
            .output()?
            .stdout;

        let output_str: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output);

        let mut recv_dropped_packets: u64 = 0;
        let mut send_dropped_packets: u64 = 0;
        let mut recv_errors_packet: u64 = 0;
        let mut send_errors_packet: u64 = 0;

        let mut line_cursor: i32 = 0;

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if line_cursor == 6 {
                (recv_dropped_packets, send_dropped_packets) = self.get_socket_info_parsing(&parts);
            }

            if line_cursor == 8 {
                (recv_errors_packet, send_errors_packet) = self.get_socket_info_parsing(&parts);
            }

            line_cursor += 1;
        }

        let network_packet_info: NetworkPacketInfo = NetworkPacketInfo::new(
            recv_dropped_packets,
            send_dropped_packets,
            recv_errors_packet,
            send_errors_packet,
        );

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
    
    #[doc = "Elasticsearch 관련 프로세스가 메모리를 얼마나 사용하는지 체크해주는 함수"]
    fn get_process_mem_usage(&mut self) -> Result<OsMemRes, anyhow::Error> {

        let target_keywords: [&str; 3] = ["java", "jdk", "elasticsearch"];

        self.system.refresh_all();

        let mut total_rss_byte: u64 = 0;
        let mut total_vms_byte: u64 = 0;

        for (_pid, proc_) in self.system.processes() {
            let name_lower: String = proc_.name().to_lowercase();

            if target_keywords.iter().any(|kw| name_lower.contains(&kw.to_lowercase())) {
                /* sysinfo: memory()와 virtual_memory()는 KiB 단위 */ 
                total_rss_byte += proc_.memory();
                total_vms_byte += proc_.virtual_memory();
            }
        }
        
        Ok(OsMemRes::new(total_rss_byte, total_vms_byte))
    }
}
