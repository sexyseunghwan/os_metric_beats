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

    
    let test = local_ip().unwrap();

    println!("{:?}", test);

}
