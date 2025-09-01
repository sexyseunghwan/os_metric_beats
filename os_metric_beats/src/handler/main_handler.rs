use crate::common::*;

use crate::repository::es_repository::*;

use crate::traits::{metirc_service::*, request_service::*, memory_service::*};

use crate::model::metric_info::*;
use crate::model::network_packet::network_packet_info::*;
use crate::model::network::network_socket_info::*;
use crate::model::network::network_usage::*;
use crate::model::system_config::*;
use crate::model::memory::os_mem_res::*;

use crate::utils_module::io_utils::*;
use crate::utils_module::time_utils::*;

use crate::env_configuration::env_config::*;

pub struct MainHandler<M: MetricService, R: RequestService, MS: MemoryService> {
    metric_service: M,
    request_service: R,
    memory_service: MS,
    private_ip: String,
}

impl<M: MetricService, R: RequestService, MS: MemoryService> MainHandler<M, R, MS> {
    pub fn new(metric_service: M, request_service: R, memory_service: MS) -> Self {
        let private_ip: String = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_err) => match read_toml_from_file::<SystemConfig>(&SYSTEM_INFO) {
                Ok(os_json) => os_json.os_server_ip,
                Err(err) => {
                    error!("{:?}", err);
                    panic!("{:?}", err)
                }
            },
        };

        Self {
            metric_service,
            request_service,
            memory_service,
            private_ip,
        }
    }

    #[doc = "시스템상의 지표를 수집해주는 함수."]
    pub async fn task_set(&mut self) -> Result<(), anyhow::Error> {
        let cur_utc_time: NaiveDateTime = get_currnet_utc_naivedatetime();
        let cur_utc_time_str: String =
            get_str_from_naivedatetime(cur_utc_time, "%Y-%m-%dT%H:%M:%SZ")?;

        let es_conn: Arc<EsRepositoryPub> = get_elastic_conn();
        
        /* 각 metric 값 호출 */
        let system_cpu_usage: f32 = self.metric_service.get_cpu_usage();
        let system_disk_usage: f64 = self.metric_service.get_disk_usage();
        let system_memory_usage: f64 = self.metric_service.get_memory_usage();
        let system_network_usage: NetworkUsage = self.metric_service.get_network_usage()?;
        let process_count: usize = self.metric_service.get_process_count();
        let network_packet_info: NetworkPacketInfo =
            self.metric_service.get_network_packet_infos()?;
        let network_socket_info: NetworkSocketInfo = self.metric_service.get_socket_info()?;
        
        /* Elasticsearch 관련 메모리 사용 지표 수집 */
        let process_mem_total: OsMemRes = self.memory_service.get_process_mem_usage()?;
        let process_use_mem: u64 = process_mem_total.working_set_size;
        let process_virtual_mem: u64 = process_mem_total.virtual_size;
        
        let log_index_name: &String = es_conn.index_pattern();

        let index_name: String = format!(
            "{}{}",
            log_index_name,
            get_str_from_naivedatetime(cur_utc_time, "%Y%m%d")?
        );

        let metric_info: MetricInfo = MetricInfo::new(
            cur_utc_time_str,
            self.private_ip.clone(),
            system_cpu_usage,
            system_disk_usage,
            system_memory_usage,
            system_network_usage.network_received,
            system_network_usage.network_transmitted,
            process_count,
            network_packet_info.recv_dropped_packets,
            network_packet_info.send_dropped_packets,
            network_packet_info.recv_errors_packet,
            network_packet_info.send_errors_packet,
            network_socket_info.tcp_connections,
            network_socket_info.udp_sockets,
            network_socket_info.tcp_established,
            network_socket_info.tcp_timewait,
            network_socket_info.tcp_listen,
            network_socket_info.tcp_close_wait,
            process_use_mem,
            process_virtual_mem,
        );

        self.request_service
            .request_metric_to_elastic(index_name, metric_info)
            .await?;

        info!("System metrics collection completed successfully.");

        Ok(())
    }
}
