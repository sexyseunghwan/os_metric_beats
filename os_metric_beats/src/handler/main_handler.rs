use crate::common::*;

use crate::repository::es_repository::get_elastic_conn;
use crate::service::os_metirc_service::*;
use crate::service::request_service::*;

use crate::model::metric_info::*;
use crate::model::system_config::*;
//use crate::model::IndexPattern::*;
//use crate::model::OsJson::*;

use crate::utils_module::io_utils::*;
use crate::utils_module::time_utils::*;

pub struct MainHandler<M: MetricService, R: RequestService> {
    metric_service: M,
    request_service: R,
    private_ip: String,
}

impl<M: MetricService, R: RequestService> MainHandler<M, R> {
    pub fn new(metric_service: M, request_service: R) -> Self {
        // let private_ip = match local_ip() {
        //     Ok(ip) => ip.to_string(),
        //     Err(_err) => match read_json_from_file::<OsJson>("./datas/os_server_info.json") {
        //         Ok(os_json) => os_json.os_server_ip,
        //         Err(err) => {
        //             error!("{:?}", err);
        //             panic!("{:?}", err)
        //         }
        //     },
        // };

        let private_ip = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_err) => match read_toml_from_file::<SystemConfig>(SYSTEM_INFO) {
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
            private_ip,
        }
    }

    #[doc = "시스템상의 지표를 수집해주는 함수."]
    pub async fn task_set(&mut self) -> Result<(), anyhow::Error> {
        let cur_utc_time = get_currnet_utc_naivedatetime();
        let cur_utc_time_str = get_str_from_naivedatetime(cur_utc_time, "%Y-%m-%dT%H:%M:%SZ")?;

        let es_conn = get_elastic_conn();

        /* 각 metric 값 호출 */
        let system_cpu_usage = self.metric_service.get_cpu_usage();
        let system_disk_usage = self.metric_service.get_disk_usage();
        let system_memory_usage = self.metric_service.get_memory_usage();
        let system_network_usage = self.metric_service.get_network_usage();
        let log_index_name = es_conn.index_pattern();

        let index_name = format!(
            "{}{}",
            log_index_name,
            get_str_from_naivedatetime(cur_utc_time, "%Y%m%d")?
        );

        let metric_info = MetricInfo::new(
            cur_utc_time_str,
            self.private_ip.clone(),
            system_cpu_usage,
            system_disk_usage,
            system_memory_usage,
            system_network_usage.network_received,
            system_network_usage.network_transmitted,
        );

        self.request_service
            .request_metric_to_elastic(index_name, metric_info)
            .await?;

        info!("System metrics collection completed successfully.");

        Ok(())
    }
}
