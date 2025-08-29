use crate::common::*;

use crate::env_configuration::env_config::*;

use crate::model::linux_config::*;
use crate::model::network::net_state::*;
use crate::model::network::network_packet_info::*;
use crate::model::network::network_socket_info::*;
use crate::model::network::network_usage::*;
use crate::model::network::iface_counters::*;

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
    #[doc = "네트워크 상태 저장 json 파일을 로드해주는 함수"]
    fn load_or_create_net_state(&self, network_tx_rx_list: &Vec<String>) -> Result<NetState, anyhow::Error> {
        match read_json_from_file::<NetState>(&NETWORK_NET_INFO_JSON) {
            Ok(state) => Ok(state),
            Err(e) => {
                warn!("[WARN] Failed to load network state: {:?}, creating new state", e);
                let new_state: NetState = self.calculate_sysfs_net_infos(network_tx_rx_list);
                self.save_net_state(&new_state)?;
                Ok(new_state)
            }
        }
    }
    
    #[doc = "네트워크 상태를 json 파일로 저장해주는 함수"]
    fn save_net_state(&self, net_state: &NetState) -> Result<(), anyhow::Error> {
        let save_file: File = std::fs::File::create(&*NETWORK_NET_INFO_JSON)?;
        serde_json::to_writer_pretty(save_file, net_state)?;
        Ok(())
    }

    #[doc = "네트워크 통계정보를 얕은복사 하여 꺼내는 함수"]
    fn get_statistics_safe(&self, net_state: &NetState, key: &str) -> IfaceCounters {
        net_state.get_statistics(key).cloned().unwrap_or_else(|| {
            error!("[ERROR] The `{}` statistics value does not exist", key);
            IfaceCounters::new(0, 0)
        })
    }
    
    #[doc = "네트워크 송신/수신 정보를 집계하는 함수"]
    fn aggregate_network_totals(&self, loop_back: &IfaceCounters, ethernet: &IfaceCounters) -> (u64, u64) {
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
    fn calculate_sysfs_net_infos(
        &self,
        network_tx_rx_list: &Vec<String>,
    ) -> NetState {
        
        /* 현재 누적 사용량 계산 */
        let mut loop_back_received: u64 = 0;    // 내부 통신 수신
        let mut loop_back_transmitted: u64 = 0; // 내부 통신 송신
        let mut ethernet_received: u64 = 0;     // 외부 통신 수신
        let mut ethernet_transmitted: u64 = 0;  // 외부 통신 송신

        let mut net_state: NetState = NetState::new(get_curretn_utc_epoch());

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

        net_state.add_statistics(String::from("loop_back"), loop_back_received, loop_back_transmitted); /* 내부통신 통계 저장 */
        net_state.add_statistics(String::from("ethernet"), ethernet_received, ethernet_transmitted);    /* 외부통신 통계 저장 */

        net_state
    }

    #[doc = "이전 네트워크 상태와 현재 네트워크 상태를 비교하여 변화량(delta)을 계산하는 함수"]
    fn calculate_network_delta(&self, prev: NetState, cur: NetState) -> NetworkUsage {
        /* 이전 상태 */ 
        let prev_loop: IfaceCounters = self.get_statistics_safe(&prev, "loop_back");
        let prev_eth: IfaceCounters  = self.get_statistics_safe(&prev, "ethernet");
        let (prev_total_rx, prev_total_tx) = self.aggregate_network_totals(&prev_loop, &prev_eth);

        /* 현재 상태 */ 
        let cur_loop: IfaceCounters = self.get_statistics_safe(&cur, "loop_back");
        let cur_eth: IfaceCounters  = self.get_statistics_safe(&cur, "ethernet");
        let (cur_total_rx, cur_total_tx) = self.aggregate_network_totals(&cur_loop, &cur_eth);

        /* 변화량 계산 */ 
        let total_rx_delta: u64      = cur_total_rx - prev_total_rx;
        let total_tx_delta: u64      = cur_total_tx - prev_total_tx;
        let loop_rx_delta: u64       = cur_loop.rx - prev_loop.rx;
        let loop_tx_delta: u64       = cur_loop.tx - prev_loop.tx;
        let eth_rx_delta: u64        = cur_eth.rx  - prev_eth.rx;
        let eth_tx_delta: u64        = cur_eth.tx  - prev_eth.tx;

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


    fn get_process_memory_usage_linux(
        &self,
        keywords: &[&str],
    ) -> Result<(u64, u64), anyhow::Error> {
        use std::fs;
        use std::path::Path;

        let mut total_rss: u64 = 0u64;
        let mut total_vss: u64 = 0u64;

        let proc_dir: &Path = Path::new("/proc");
        if let Ok(entries) = fs::read_dir(proc_dir) {
            for entry in entries.flatten() {
                if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
                    let status_path: String = format!("/proc/{}/status", pid);
                    let cmdline_path: String = format!("/proc/{}/cmdline", pid);

                    if let (Ok(status_content), Ok(cmdline_content)) = (
                        fs::read_to_string(&status_path),
                        fs::read_to_string(&cmdline_path),
                    ) {
                        let cmdline_lower: String = cmdline_content.to_lowercase();
                        let matches_keyword: bool =
                            keywords.iter().any(|kw| cmdline_lower.contains(kw));

                        if matches_keyword {
                            let mut rss_kb: u64 = 0u64;
                            let mut vss_kb: u64 = 0u64;

                            for line in status_content.lines() {
                                if line.starts_with("VmRSS:") {
                                    if let Some(value) = line.split_whitespace().nth(1) {
                                        rss_kb = value.parse().unwrap_or(0);
                                    }
                                } else if line.starts_with("VmSize:") {
                                    if let Some(value) = line.split_whitespace().nth(1) {
                                        vss_kb = value.parse().unwrap_or(0);
                                    }
                                }
                            }

                            total_rss += rss_kb * 1024;
                            total_vss += vss_kb * 1024;
                        }
                    }
                }
            }
        }

        Ok((total_rss, total_vss))
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

        let prev_net_state: NetState = self.load_or_create_net_state(network_tx_rx_list)?; /* 이전 네트워크 내부/외부 송/수신 데이터 계산 */
        let cur_net_state: NetState = self.calculate_sysfs_net_infos(network_tx_rx_list); /* 현재 네트워크 내부/외부 송/수신 데이터 계산 */

        /* 수집할 네트워크 사용량 지표 */
        let cur_network_usage: NetworkUsage = self.calculate_network_delta(prev_net_state, cur_net_state);
        
        Ok(cur_network_usage)
    }

    #[doc = "현재 시스템의 프로세스의 개수를 반환해주는 함수"]
    fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();
        self.system.processes().len()
    }


    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error> {

        

        let dev_content: String = fs::read_to_string("/proc/net/dev").unwrap_or_default();
        let snmp_content: String = fs::read_to_string("/proc/net/snmp").unwrap_or_default();

        let mut recv_dropped_packets: i64 = 0;
        let mut send_dropped_packets: i64 = 0;
        let mut recv_errors_packet: i64 = 0;
        let mut send_errors_packet: i64 = 0;

        for line in dev_content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 16 {
                if let (Ok(rx_dropped), Ok(tx_dropped), Ok(rx_errors), Ok(tx_errors)) = (
                    parts[4].parse::<i64>(),
                    parts[12].parse::<i64>(),
                    parts[3].parse::<i64>(),
                    parts[11].parse::<i64>(),
                ) {
                    recv_dropped_packets += rx_dropped;
                    send_dropped_packets += tx_dropped;
                    recv_errors_packet += rx_errors;
                    send_errors_packet += tx_errors;
                }
            }
        }

        let network_packet_info: NetworkPacketInfo = NetworkPacketInfo::new(
            recv_dropped_packets,
            send_dropped_packets,
            recv_errors_packet,
            send_errors_packet,
        );

        Ok(network_packet_info)
    }


    fn get_socket_info_parsing(&mut self, socket_vec: &Vec<&str>) -> (i64, i64) {
        let recv_packet: i64 = match socket_vec.get(socket_vec.len() - 2) {
            Some(recv_packet) => recv_packet.parse::<i64>().unwrap_or(0),
            _none => {
                error!("[Error][get_socket_info_parsing()] The value 'recv_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        let send_packet: i64 = match socket_vec.last() {
            Some(send_packet) => send_packet.parse::<i64>().unwrap_or(0),
            _none => {
                error!("[Error][get_socket_info_parsing()] The value 'send_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        (recv_packet, send_packet)
    }



    fn get_socket_info(&mut self) -> Result<NetworkSocketInfo, anyhow::Error> {
        use std::fs;

        let tcp_content: String = fs::read_to_string("/proc/net/tcp").unwrap_or_default();
        let tcp6_content: String = fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
        let udp_content: String = fs::read_to_string("/proc/net/udp").unwrap_or_default();
        let udp6_content: String = fs::read_to_string("/proc/net/udp6").unwrap_or_default();

        let mut tcp_connections: i32 = 0i32;
        let mut udp_sockets: i32 = 0i32;
        let mut tcp_established: i32 = 0i32;
        let mut tcp_timewait: i32 = 0i32;
        let mut tcp_listen: i32 = 0i32;
        let mut tcp_close_wait: i32 = 0i32;

        for content in [tcp_content, tcp6_content].iter() {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    tcp_connections += 1;
                    if let Ok(state) = u32::from_str_radix(parts[3], 16) {
                        match state {
                            0x01 => tcp_established += 1,
                            0x0A => tcp_listen += 1,
                            0x06 => tcp_timewait += 1,
                            0x08 => tcp_close_wait += 1,
                            _ => {}
                        }
                    }
                }
            }
        }

        for content in [udp_content, udp6_content].iter() {
            for line in content.lines().skip(1) {
                udp_sockets += 1;
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
