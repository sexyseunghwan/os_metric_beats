use crate::common::*;


pub trait MetricService {
    fn get_cpu_usage(&mut self) -> f32;
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

    fn get_cpu_usage(&mut self) -> f32 {

        // 시스템 정보를 새로 고침 (CPU 사용량 등을 업데이트)
        self.system.refresh_cpu();

        let cpu_usage_sum: f32 = self.system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let cpu_thread_cnt = self.system.cpus().len();

        if cpu_thread_cnt == 0 {
            return 0.0; 
        }

        let cpu_usage_avg = cpu_usage_sum / cpu_thread_cnt as f32;
        
        cpu_usage_avg
    }

}