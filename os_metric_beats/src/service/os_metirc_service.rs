use crate::common::*;

use crate::model::NetworkUsage::*;

pub trait MetricService {
    fn get_cpu_usage(&mut self) -> f32;
    fn get_cpu_usage_avg_thread(&mut self) -> f32;
    fn get_disk_usage(&mut self) -> f64;
    fn get_memory_usage(&mut self) -> f64;
    fn get_network_usage(&mut self) -> NetworkUsage;
}

#[derive(Debug)]
pub struct MetricServicePub {
    system: System
}

impl MetricServicePub {

    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all(); // 시스템 정보 초기화
        MetricServicePub { system }
    }
}


impl MetricService for MetricServicePub {

    
    #[doc = "cpu 의 사용률을 체크. - cpu Max 값 추출"]
    fn get_cpu_usage(&mut self) -> f32 {

        /* 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트) */ 
        self.system.refresh_cpu();
        
        let mut max_cpu_val: f32 = 0.0;
        
        for cpu in self.system.cpus() {
            let thread_cpu_usage = cpu.cpu_usage();
            max_cpu_val = max_cpu_val.max(thread_cpu_usage);
        }

        max_cpu_val.round() * 100.0 / 100.0

    }


    #[doc = "cpu 의 평균 사용률을 체크. - 스레드별 평균"]
    fn get_cpu_usage_avg_thread(&mut self) -> f32 {

        /* 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트) */ 
        self.system.refresh_cpu();

        let cpu_usage_sum: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let cpu_thread_cnt = self.system.cpus().len();

        if cpu_thread_cnt == 0 {
            return 0.0; 
        }

        let cpu_usage_avg = cpu_usage_sum / cpu_thread_cnt as f32;
        let cpu_usage_avg_round = cpu_usage_avg.round() * 100.0 / 100.0;

        cpu_usage_avg_round
    }


    
    #[doc = "disk 사용률을 체크"]
    fn get_disk_usage(&mut self) -> f64 {

        self.system.refresh_disks_list();

        if let Some(disk) = self.system.disks().iter().next() {
            let total_space = disk.total_space() as f64;
            let available_space = disk.available_space() as f64;
            let used_space = total_space - available_space;

            let usage_percentage = (used_space / total_space) * 100.0;
            return (usage_percentage * 100.0).round() / 100.0
        }

        0.0
    }

    
    
    #[doc = "memory 사용률을 체크"]
    fn get_memory_usage(&mut self) -> f64 {
        
        self.system.refresh_memory();

        let total_memory = self.system.total_memory() as f64;
        let used_memory = self.system.used_memory() as f64;

        // 사용된 메모리 비율 계산
        let usage_percentage = (used_memory / total_memory) * 100.0;

        // 소수점 둘째 자리에서 반올림
        (usage_percentage * 100.0).round() / 100.0
    }
    
    
    
    #[doc = "Network 사용량 체크"]
    fn get_network_usage(&mut self) -> NetworkUsage {

        self.system.refresh_networks_list();

        let networks = self.system.networks();
        let mut network_received: u64 = 0;
        let mut network_transmitted: u64 = 0;

        for (_interface_name, network) in networks.iter() {
            network_received += network.received();
            network_transmitted += network.transmitted();
        }
        
        NetworkUsage::new(network_received, network_transmitted)
    }
    
}