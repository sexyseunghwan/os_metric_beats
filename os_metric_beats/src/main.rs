pub mod common;
use common::*;

pub mod handler;
pub mod repository;
pub mod service;
pub mod model;

pub mod utils_module;
use utils_module::logger_utils::*;

#[tokio::main]
async fn main() {
    
    set_global_logger();
    info!("program start");


    let mut system = System::new_all();

    // 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트)
    system.refresh_all();

    // 모든 CPU의 사용량을 출력
    for cpu in system.cpus() {
        println!("CPU: {}%", cpu.cpu_usage());
    }

}
