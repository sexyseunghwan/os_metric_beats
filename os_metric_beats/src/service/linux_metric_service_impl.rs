use crate::common::*;

use crate::common_enums::tcp_state::TcpState;

use crate::env_configuration::env_config::*;

use crate::model::{
    linux_config::*,
    network::{
        iface_counters::*, net_state::*, network_socket_info::*,
        network_usage::*,
    },
    network_packet::{packet_state::*, network_packet_info::*},
    memory::os_mem_res::*
};

use crate::traits::metirc_service::*;

use crate::utils_module::io_utils::*;
use crate::utils_module::math_utils::*;
use crate::utils_module::time_utils::*;

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct LinuxMetricServiceImpl {
    system: System,
    linux_config: LinuxConfig,
}

impl LinuxMetricServiceImpl {
    pub fn new() -> Self {
        /* Linux 전용 설정 데이터 불러오기 */
        let linux_config: LinuxConfig = read_toml_from_file::<LinuxConfig>(&LINUX_CONFIG_INFO)
            .unwrap_or_else(|e| {
                error!("[ERROR][LinuxMetricServiceImpl->new] {:?}", e);
                panic!("[ERROR][LinuxMetricServiceImpl->new] {:?}", e);
            });

        let mut system: System = System::new_all();
        system.refresh_all();

        LinuxMetricServiceImpl {
            system,
            linux_config,
        }
    }

    /*========================================================================================*/
    /*=================================== NETWORK NET STATE===================================*/
    /*========================================================================================*/
    #[doc = "네트워크 통계정보를 얕은복사 하여 꺼내는 함수"]
    fn get_statistics_safe(&self, net_state: &NetState, key: &str) -> IfaceCounters {
        net_state.get_statistics(key).cloned().unwrap_or_else(|| {
            error!("[ERROR] The `{}` statistics value does not exist", key);
            IfaceCounters::new(0, 0)
        })
    }

    #[doc = "네트워크 송신/수신 정보를 집계하는 함수"]
    fn aggregate_network_totals(
        &self,
        loop_back: &IfaceCounters,
        ethernet: &IfaceCounters,
    ) -> (u64, u64) {
        let total_received: u64 = loop_back.rx + ethernet.rx;
        let total_transmitted: u64 = loop_back.tx + ethernet.tx;
        (total_received, total_transmitted)
    }

    #[doc = "네트워크 송신/수신량 데이터를 수집해준다."]
    fn read_sysfs_net_totals(&self, iface: &str) -> Result<(u64, u64), anyhow::Error> {
        let rx: u64 = read_u64(format!("/sys/class/net/{}/statistics/rx_bytes", iface))?;
        let tx: u64 = read_u64(format!("/sys/class/net/{}/statistics/tx_bytes", iface))?;
        Ok((rx, tx))
    }

    #[doc = "linux의 system network 정보를 계산해주는 함수 - network 송/수신 상태를 계산"]
    fn calculate_sysfs_net_infos(&self, network_tx_rx_list: &Vec<String>) -> NetState {
        /* 현재 누적 사용량 계산 */
        let mut loop_back_received: u64 = 0; // 내부 통신 수신
        let mut loop_back_transmitted: u64 = 0; // 내부 통신 송신
        let mut ethernet_received: u64 = 0; // 외부 통신 수신
        let mut ethernet_transmitted: u64 = 0; // 외부 통신 송신

        let mut net_state: NetState = NetState::new(get_currnet_utc_str());

        for iface in network_tx_rx_list {
            if let Ok((rx, tx)) = self.read_sysfs_net_totals(iface.as_str()) {
                net_state.add_iface(iface.to_string(), rx, tx);

                if iface.starts_with("l") {
                    /* 내부 통신 */
                    loop_back_received += rx;
                    loop_back_transmitted += tx;
                } else {
                    /* 외부 통신 */
                    ethernet_received += rx;
                    ethernet_transmitted += tx;
                }
            } else {
                warn!("[WARN][LinuxMetricServiceImpl->calculate_sysfs_net_infos] read_sysfs_net_totals failed for iface={}", iface);
            }
        }

        net_state.add_statistics(
            String::from("loop_back"),
            loop_back_received,
            loop_back_transmitted,
        ); /* 내부통신 통계 저장 */
        net_state.add_statistics(
            String::from("ethernet"),
            ethernet_received,
            ethernet_transmitted,
        ); /* 외부통신 통계 저장 */

        net_state
    }
    
    #[doc = "이전 네트워크 상태와 현재 네트워크 상태를 비교하여 변화량(delta)을 계산하는 함수"]
    fn calculate_network_delta(&self, prev: NetState, cur: NetState) -> NetworkUsage {
        /* 이전 상태 */
        let prev_loop: IfaceCounters = self.get_statistics_safe(&prev, "loop_back");
        let prev_eth: IfaceCounters = self.get_statistics_safe(&prev, "ethernet");
        let (prev_total_rx, prev_total_tx) = self.aggregate_network_totals(&prev_loop, &prev_eth);

        /* 현재 상태 */
        let cur_loop: IfaceCounters = self.get_statistics_safe(&cur, "loop_back");
        let cur_eth: IfaceCounters = self.get_statistics_safe(&cur, "ethernet");
        let (cur_total_rx, cur_total_tx) = self.aggregate_network_totals(&cur_loop, &cur_eth);

        /* 변화량 계산 */
        let total_rx_delta: u64 = cur_total_rx - prev_total_rx;
        let total_tx_delta: u64 = cur_total_tx - prev_total_tx;
        let loop_rx_delta: u64 = cur_loop.rx - prev_loop.rx;
        let loop_tx_delta: u64 = cur_loop.tx - prev_loop.tx;
        let eth_rx_delta: u64 = cur_eth.rx - prev_eth.rx;
        let eth_tx_delta: u64 = cur_eth.tx - prev_eth.tx;
        
        /* 결과 반환 */
        NetworkUsage::new(
            total_rx_delta,
            total_tx_delta,
            loop_rx_delta,
            loop_tx_delta,
            eth_rx_delta,
            eth_tx_delta,
        )
    }

    /*======================================================================================*/
    /*=================================== NETWORK PACKET ===================================*/
    /*======================================================================================*/
    #[doc = "네트워크 송신/수신 관련 패킷 정보를 수집해준다."]
    fn read_sysfs_packt_totals(&self, iface: &str) -> Result<NetworkPacketInfo, anyhow::Error> {
        let recv_dropped_packets: u64 =
            read_u64(format!("/sys/class/net/{}/statistics/rx_dropped", iface))?;
        let send_dropped_packets: u64 =
            read_u64(format!("/sys/class/net/{}/statistics/tx_dropped", iface))?;
        let recv_errors_packet: u64 =
            read_u64(format!("/sys/class/net/{}/statistics/rx_errors", iface))?;
        let send_errors_packet: u64 =
            read_u64(format!("/sys/class/net/{}/statistics/tx_errors", iface))?;

        let network_packet: NetworkPacketInfo = NetworkPacketInfo::new(
            recv_dropped_packets,
            send_dropped_packets,
            recv_errors_packet,
            send_errors_packet,
        );
        
        Ok(network_packet)
    }

    
    #[doc = "linux의 system network packet 정보를 계산해주는 함수"]
    fn calculate_sysfs_packet_infos(&self, network_tx_rx_list: &Vec<String>) -> PacketState {

        let mut network_packet_info: PacketState = PacketState::new(get_currnet_utc_str());
        
        for iface in network_tx_rx_list {
            if let Ok(network_packet) = self.read_sysfs_packt_totals(iface.as_str()) {
                network_packet_info.add_iface(iface.to_string(), network_packet)
            } else {
                warn!("[WARN][LinuxMetricServiceImpl->calculate_sysfs_packet_infos] read_sysfs_net_totals failed for iface={}", iface);
            }
        }
        
        network_packet_info
    }

    #[doc = "이전 네트워크 패킷 상태와 현재 네트워크 패킷 상태를 비교하여 변화량(delta)을 계산하는 함수"]
    fn calculate_network_packet_delta(&self, prev: PacketState, cur: PacketState, network_tx_rx_list: &Vec<String>) -> NetworkPacketInfo {

        let mut recv_dropped_packets: u64 = 0; 
        let mut send_dropped_packets: u64 = 0; 
        let mut recv_errors_packet: u64 = 0; 
        let mut send_errors_packet: u64 = 0; 
        
        for network in network_tx_rx_list {
            
            let prev_iface_counter = prev.get_iface(network);
            let cur_iface_counter = cur.get_iface(network);

            if let(Some(prev_c), Some(cur_c)) = (prev_iface_counter, cur_iface_counter) {
                recv_dropped_packets += cur_c.rx_dropped.saturating_sub(prev_c.rx_dropped);
                send_dropped_packets += cur_c.tx_dropped.saturating_sub(prev_c.tx_dropped);
                recv_errors_packet += cur_c.rx_errors.saturating_sub(prev_c.rx_errors);
                send_errors_packet += cur_c.tx_errors.saturating_sub(prev_c.tx_errors);
            } else {
                error!(
                    "[ERROR][LinuxMetricServiceImpl->calculate_network_packet_delta] missing iface counter: prev={:?}, cur={:?}, iface={}",
                    prev_iface_counter, cur_iface_counter, network
                );
            }
        }   

        NetworkPacketInfo::new(recv_dropped_packets, send_dropped_packets, recv_errors_packet, send_errors_packet)
    }
    

    /*======================================================================================*/
    /*===================================== SOCKET INFO ====================================*/
    /*======================================================================================*/

    
    #[doc = "네트워크 파일을 안전하게 읽는 헬퍼 함수"]
    fn read_network_file(&self, path: &str) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }
    
    #[doc = "TCP 연결 정보를 파싱하는 헬퍼 함수"]
    fn parse_tcp_connections(&self, content: &str) -> (i32, i32, i32, i32, i32) {
        let mut connections: i32 = 0;
        let mut established: i32 = 0;
        let mut timewait: i32 = 0;
        let mut listen: i32 = 0;
        let mut close_wait: i32 = 0;
        
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                connections += 1;
                if let Ok(state_value) = u32::from_str_radix(parts[3], 16) {
                    if let Some(tcp_state) = TcpState::from_u32(state_value) {
                        match tcp_state {
                            TcpState::Established => established += 1,
                            TcpState::Listen => listen += 1,
                            TcpState::TimeWait => timewait += 1,
                            TcpState::CloseWait => close_wait += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        (connections, established, timewait, listen, close_wait)
    }
    
    #[doc = "UDP 소켓 개수를 계산하는 헬퍼 함수"]
    fn count_udp_sockets(&self, content: &str) -> i32 {
        content.lines().skip(1).count() as i32
    }
}

impl MetricService for LinuxMetricServiceImpl {
    #[doc = "CPU 사용률을 수집해주는 함수"]
    fn get_cpu_usage(&mut self) -> f32 {
        self.system.refresh_cpu();

        let max: f32 = self
            .system
            .cpus()
            .iter()
            .map(|c| c.cpu_usage()) /* 각 논리 코어 사용률 (0.0 ~ 100.0) */
            .fold(0.0_f32, f32::max);

        round2(max.clamp(0.0, 100.0))
    }

    #[doc = "CPU 각 스레드의 평균 사용률을 수집해주는 함수"]
    fn get_cpu_usage_avg_thread(&mut self) -> f32 {
        self.system.refresh_cpu();

        let cpu_usage_sum: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let cpu_thread_cnt: usize = self.system.cpus().len();

        if cpu_thread_cnt == 0 {
            return 0.0;
        }

        let cpu_usage_avg: f32 = cpu_usage_sum / cpu_thread_cnt as f32;
        round2(cpu_usage_avg.clamp(0.0, 100.0))
    }

    #[doc = "마운트된 디스크 사용률을 수집해주는 함수"]
    fn get_disk_usage(&mut self) -> f64 {
        self.system.refresh_disks_list();

        if let Some(disk) = self.system.disks().iter().find(|d| {
            d.mount_point()
                .to_str()
                .map_or(false, |path| path == "/" || path.starts_with("/data"))
        }) {
            let total_space: f64 = disk.total_space() as f64;
            let available_space: f64 = disk.available_space() as f64;
            let used_space: f64 = total_space - available_space;
            let usage_percentage: f64 = (used_space / total_space) * 100.0;

            return round2_f64(usage_percentage.clamp(0.0, 100.0));
        }

        if let Some(disk) = self.system.disks().iter().next() {
            let total_space: f64 = disk.total_space() as f64;
            let available_space: f64 = disk.available_space() as f64;
            let used_space: f64 = total_space - available_space;
            let usage_percentage: f64 = (used_space / total_space) * 100.0;

            return round2_f64(usage_percentage.clamp(0.0, 100.0));
        }

        0.0
    }

    #[doc = "시스템 메모리 사용률을 수집해주는 함수"]
    fn get_memory_usage(&mut self) -> f64 {
        self.system.refresh_memory();

        let total_memory: f64 = self.system.total_memory() as f64;
        let used_memory: f64 = self.system.used_memory() as f64;

        let usage_percentage: f64 = (used_memory / total_memory) * 100.0;

        round2_f64(usage_percentage.clamp(0.0, 100.0))
    }

    #[doc = "네트워크 사용량 데이터를 수집하고 반환하는 함수"]
    fn get_network_usage(&mut self) -> Result<NetworkUsage, anyhow::Error> {
        let linux_config: &LinuxConfig = self.linux_config();
        let network_tx_rx_list: &Vec<String> = linux_config.network_tx_rx_list();

        /* 이전 네트워크 내부/외부 송/수신 데이터 계산 */
        let prev_net_state: NetState =
            load_or_create_file(network_tx_rx_list, &NETWORK_NET_INFO_JSON, |list| {
                self.calculate_sysfs_net_infos(list)
            })?;
         
        let cur_net_state: NetState = self.calculate_sysfs_net_infos(network_tx_rx_list); /* 현재 네트워크 내부/외부 송/수신 데이터 계산 */
        
        /* 현재 네트워크 지표를 파일에 써준다. */
        save_as_json::<NetState>(&cur_net_state, &*NETWORK_NET_INFO_JSON)?;

        /* 수집할 네트워크 사용량 지표 */
        let cur_network_usage: NetworkUsage =
            self.calculate_network_delta(prev_net_state, cur_net_state);
        
        Ok(cur_network_usage)
    }

    #[doc = "현재 시스템의 프로세스의 개수를 반환해주는 함수"]
    fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();
        self.system.processes().len()
    }

    #[doc = "네트워크 패킷정보를 계산해주는 함수"]
    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error> {
        let linux_config: &LinuxConfig = self.linux_config();
        let network_tx_rx_list: &Vec<String> = linux_config.network_tx_rx_list();

        /* 이전 네트워크의 패킷 데이터 계산 */
        let prev_packet_state: PacketState = load_or_create_file(network_tx_rx_list, &NETWORK_PACKET_INFO_JSON, |list| {
            self.calculate_sysfs_packet_infos(list)
        })?;
        
        let cur_packet_state: PacketState = self.calculate_sysfs_packet_infos(network_tx_rx_list); /* 현재 네트워크 패킷 데이터 계산 */
        
        /* 현재 네트워크 패킷 지표를 파일에 써준다. */
        save_as_json::<PacketState>(&cur_packet_state, &NETWORK_PACKET_INFO_JSON)?;
        
        /* 수집할 네트워크 사용량 지표 */
        let network_packet_usage: NetworkPacketInfo = self.calculate_network_packet_delta(prev_packet_state, cur_packet_state, network_tx_rx_list);
        
        Ok(network_packet_usage)
    }

    // 이건 필요 없는 듯 해보이는데?
    fn get_socket_info_parsing(&mut self, socket_vec: &Vec<&str>) -> (u64, u64) {
        let recv_packet: u64 = match socket_vec.get(socket_vec.len() - 2) {
            Some(recv_packet) => recv_packet.parse::<u64>().unwrap_or(0),
            _none => {
                error!("[Error][get_socket_info_parsing()] The value 'recv_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        let send_packet: u64 = match socket_vec.last() {
            Some(send_packet) => send_packet.parse::<u64>().unwrap_or(0),
            _none => {
                error!("[Error][get_socket_info_parsing()] The value 'send_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        (recv_packet, send_packet)
    }

    #[doc = "System 의 소켓 정보를 반환해주는 함수"]
    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error> {
        
        /* 네트워크 파일 읽기 */ 
        let tcp_content: String = self.read_network_file("/proc/net/tcp");
        let tcp6_content: String = self.read_network_file("/proc/net/tcp6");
        let udp_content: String = self.read_network_file("/proc/net/udp");
        let udp6_content: String = self.read_network_file("/proc/net/udp6");

        /* TCP 연결 정보 파싱 */ 
        let (tcp_conn, tcp_est, tcp_tw, tcp_listen, tcp_cw) = 
            self.parse_tcp_connections(&tcp_content);
        let (tcp6_conn, tcp6_est, tcp6_tw, tcp6_listen, tcp6_cw) = 
            self.parse_tcp_connections(&tcp6_content);

        /* UDP 소켓 수 계산 */ 
        let udp4_sockets: i32 = self.count_udp_sockets(&udp_content);
        let udp6_sockets: i32 = self.count_udp_sockets(&udp6_content);

        /* 전체 통계 집계 */ 
        let total_tcp_connections: i32 = tcp_conn + tcp6_conn;
        let total_udp_sockets: i32 = udp4_sockets + udp6_sockets;
        let total_tcp_established: i32 = tcp_est + tcp6_est;
        let total_tcp_timewait: i32 = tcp_tw + tcp6_tw;
        let total_tcp_listen: i32 = tcp_listen + tcp6_listen;
        let total_tcp_close_wait: i32 = tcp_cw + tcp6_cw;
        
        Ok(NetworkSocketInfo::new(
            total_tcp_connections,
            total_udp_sockets,
            total_tcp_established,
            total_tcp_timewait,
            total_tcp_listen,
            total_tcp_close_wait,
        ))
    }
    
    #[doc = ""]
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
