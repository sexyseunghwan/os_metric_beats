use crate::common::*;
use crate::model::network_packet_info::*;
use crate::model::network_socket_info::*;
use crate::model::network_usage::*;
use crate::traits::metirc_service::*;

#[derive(Debug)]
pub struct LinuxMetricServiceImpl {
    system: System,
}

impl LinuxMetricServiceImpl {
    pub fn new() -> Self {
        let mut system: System = System::new_all();
        system.refresh_all();
        LinuxMetricServiceImpl { system }
    }

    #[doc = "네트워크 인터페이스별 통계 메트릭을 수집해주는 함수"]
    fn read_proc_net_dev(&self) -> Result<(u64, u64), std::io::Error> {
        let contents: String = fs::read_to_string("/proc/net/dev")?;

        let mut total_rx_bytes: u64 = 0u64;
        let mut total_tx_bytes: u64 = 0u64;

        for line in contents.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                if let (Ok(rx_bytes), Ok(tx_bytes)) =
                    (parts[1].parse::<u64>(), parts[9].parse::<u64>())
                {
                    total_rx_bytes += rx_bytes;
                    total_tx_bytes += tx_bytes;
                }
            }
        }

        Ok((total_rx_bytes, total_tx_bytes))
    }

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
    
    #[doc = "CPU 사용률을 수집해주는 함수 - Linux 용도"]
    fn get_cpu_usage(&mut self) -> f32 {
        self.system.refresh_cpu();

        let mut max_cpu_val: f32 = 0.0;
        for cpu in self.system.cpus() {
            let thread_cpu_usage: f32 = cpu.cpu_usage();
            max_cpu_val = max_cpu_val.max(thread_cpu_usage);
        }

        max_cpu_val.round() * 100.0 / 100.0
    }

    fn get_cpu_usage_avg_thread(&mut self) -> f32 {
        self.system.refresh_cpu();

        let cpu_usage_sum: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let cpu_thread_cnt: usize = self.system.cpus().len();

        if cpu_thread_cnt == 0 {
            return 0.0;
        }

        let cpu_usage_avg: f32 = cpu_usage_sum / cpu_thread_cnt as f32;
        cpu_usage_avg.round() * 100.0 / 100.0
    }

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

            return (usage_percentage * 100.0).round() / 100.0;
        }

        if let Some(disk) = self.system.disks().iter().next() {
            let total_space: f64 = disk.total_space() as f64;
            let available_space: f64 = disk.available_space() as f64;
            let used_space: f64 = total_space - available_space;
            let usage_percentage: f64 = (used_space / total_space) * 100.0;
            return (usage_percentage * 100.0).round() / 100.0;
        }

        0.0
    }

    fn get_memory_usage(&mut self) -> f64 {
        self.system.refresh_memory();

        let total_memory: f64 = self.system.total_memory() as f64;
        let used_memory: f64 = self.system.used_memory() as f64;

        let usage_percentage: f64 = (used_memory / total_memory) * 100.0;
        (usage_percentage * 100.0).round() / 100.0
    }

    #[doc = "네트워크 사용량을 수집해주는 함수"]
    fn get_network_usage(&mut self) -> NetworkUsage {
        match self.read_proc_net_dev() {
            Ok((rx_bytes, tx_bytes)) => NetworkUsage::new(rx_bytes, tx_bytes),
            Err(_) => {
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
        }
    }

    fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();
        self.system.processes().len()
    }

    fn get_socket_info_parsing(&mut self, socket_vec: &Vec<&str>) -> (i64, i64) {
        let recv_packet: i64 = match socket_vec.get(socket_vec.len() - 2) {
            Some(recv_packet) => recv_packet.parse::<i64>().unwrap_or(0),
            None => {
                error!("[Error][get_socket_info_parsing()] The value 'recv_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        let send_packet: i64 = match socket_vec.last() {
            Some(send_packet) => send_packet.parse::<i64>().unwrap_or(0),
            None => {
                error!("[Error][get_socket_info_parsing()] The value 'send_packet' does not exist. : {:?}", socket_vec);
                0
            }
        };

        (recv_packet, send_packet)
    }

    fn get_network_packet_infos(&mut self) -> Result<NetworkPacketInfo, anyhow::Error> {
        use std::fs;

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

        let network_packet_info = NetworkPacketInfo::new(
            recv_dropped_packets,
            send_dropped_packets,
            recv_errors_packet,
            send_errors_packet,
        );

        Ok(network_packet_info)
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
