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
              2025-08-00 Seunghwan Shin       # [v.2.0.0] Linux & Windows 호환해서 사용할 수 있도록 프로그램 전체 수정
*/

pub mod common;
use common::*;

pub mod handler;
use handler::main_handler::*;

pub mod repository;

pub mod service;
use service::linux_metric_service_impl::*;
use service::linux_process_service_impl::*;
use service::request_service_impl::*;
use service::windows_metirc_service_impl::*;
use service::wmi_conn_service_impl::*;

pub mod model;
use model::system_config::*;

pub mod utils_module;
use utils_module::io_utils::*;
use utils_module::logger_utils::*;

pub mod env_configuration;
use env_configuration::env_config::*;

pub mod traits;

#[tokio::main]
async fn main() {
    dotenv().ok();
    set_global_logger();

    info!("Operating System Metricbeats Program Start");

    let system_config: SystemConfig = match read_toml_from_file::<SystemConfig>(&SYSTEM_INFO) {
        Ok(config) => config,
        Err(err) => {
            error!("[ERROR][main] Failed to read system config: {:?}", err);
            panic!("[ERROR][main] Failed to read system config: {:?}", err);
        }
    };

    /* Operation System 별로 구분 -> Windows/Linux */
    let os_ver: String = system_config.os_ver.to_lowercase();

    if os_ver == "windows" {
        run_windows_mode().await;
    } else if os_ver == "linux" {
        run_linux_mode().await;
    } else {
        error!("[ERROR][main] Unsupported OS version: {}", os_ver);
        panic!("[ERROR][main] Unsupported OS version: {}", os_ver);
    }
}

#[doc = "Windows OS 전용 모드"]
async fn run_windows_mode() {
    info!("Running in Windows mode");

    let os_metirc_service: WindowsMetricServiceImpl = WindowsMetricServiceImpl::new();
    let request_service: RequestServiceImpl = RequestServiceImpl::new();
    let wmi_conn_service: WmiConnServiceImpl = WmiConnServiceImpl::new();
    let mut main_handler: MainHandler<
        WindowsMetricServiceImpl,
        RequestServiceImpl,
        WmiConnServiceImpl,
    > = MainHandler::new(os_metirc_service, request_service, wmi_conn_service);

    loop {
        match main_handler.task_set().await {
            Ok(_) => (),
            Err(err) => {
                error!("[ERROR][run_windows_mode] {:?}", err);
                std_sleep(Duration::from_secs(10));
                continue;
            }
        };

        std_sleep(Duration::from_secs(10));
    }
}

#[doc = "Linux OS 전용 모드"]
async fn run_linux_mode() {
    info!("Running in Linux mode");

    let os_metirc_service: LinuxMetricServiceImpl = LinuxMetricServiceImpl::new();
    let request_service: RequestServiceImpl = RequestServiceImpl::new();
    let linux_process_service: LinuxProcessServiceImpl = LinuxProcessServiceImpl::new();
    let mut main_handler: MainHandler<
        LinuxMetricServiceImpl,
        RequestServiceImpl,
        LinuxProcessServiceImpl,
    > = MainHandler::new(os_metirc_service, request_service, linux_process_service);

    loop {
        match main_handler.task_set().await {
            Ok(_) => (),
            Err(err) => {
                error!("[ERROR][run_linux_mode] {:?}", err);
                std_sleep(Duration::from_secs(10));
                continue;
            }
        };

        std_sleep(Duration::from_secs(10));
    }
}
