/*
Author      : Seunghwan Shin
Create date : 2024-10-16
Description : Elasticsearch 가 실행되고 있는 OS 의 metric 정보를 수집하기 위한 프로그램.

History     : 2024-10-16 Seunghwan Shin       # [v.1.0.0] first create
              2024-10-21 Seunghwan Shin       # [v.1.1.0]
                                                1) 시스템 메트릭을 저장할 인덱스를 동적으로 바꿀수 있도록 소스변경.
                                                2) Network 메트릭도 시스템 모니터 메트릭 대상으로 추가.
              2024-12-02 Seunghwan Shin       # [v.1.1.1] CPU 모니터링을 스레드 평균에서 max 값으로 변경
              2025-01-08 Seunghwan Shin       # [v.1.1.2] 설정파일을 모두 json -> toml 파일로 전환
              2025-02-12 Seunghwan Shin       # [v.1.2.0] .env 파일사용으로 경로변경을 쉽게 할 수 있도록 변경
              2025-07-18 Seunghwan Shin       # [v.1.3.0] off-heap 사용량 추적을 위해서, wmi 지표 추가 수집
*/

pub mod common;
use common::*;

pub mod handler;
use handler::main_handler::*;

pub mod repository;

pub mod service;
use service::os_metirc_service_impl::*;
use service::request_service_impl::*;
use service::wmi_conn_service_impl::*;

pub mod model;

pub mod utils_module;
use utils_module::logger_utils::*;

pub mod env_configuration;

pub mod traits;


#[tokio::main]
async fn main() {
    set_global_logger();
    info!("Operating System Metricbeats Program Start");

    let os_metirc_service: MetricServiceImpl = MetricServiceImpl::new();
    let request_service: RequestServiceImpl = RequestServiceImpl::new();
    let wmi_conn_service: WmiConnServiceImpl = WmiConnServiceImpl::new();
    let mut main_handler: MainHandler<MetricServiceImpl, RequestServiceImpl, WmiConnServiceImpl> =
        MainHandler::new(os_metirc_service, request_service, wmi_conn_service);

    loop {
        match main_handler.task_set().await {
            Ok(_) => (),
            Err(err) => {
                error!("{:?}", err);
                std::thread::sleep(Duration::from_secs(10));
                continue;
            }
        };

        std::thread::sleep(Duration::from_secs(10));
    }
}
