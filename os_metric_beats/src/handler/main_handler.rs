use crate::common::*;

use crate::service::os_metirc_service::*;
use crate::service::request_service::*;

pub struct MainHandler<M: MetricService, R: RequestService> {
    metric_service: M,
    request_service: R
}

impl<M: MetricService, R: RequestService> MainHandler<M, R> {

    /*

    */
    pub fn new(metric_service: M, request_service: R) -> Self {
        Self {
            metric_service, request_service
        }
    }


    /*
    
    */
    pub async fn task_set(&mut self) -> Result<(), anyhow::Error> {


        loop {

            let cpu_avg = self.metric_service.get_cpu_usage();
            
            

            break;
            std::thread::sleep(Duration::from_secs(10));
        }


        Ok(())
    }

}