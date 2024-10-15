use crate::common::*;

use crate::repository::es_repository::*;

#[async_trait]
pub trait RequestService {
    async fn request_metric_to_elastic(cpu_usage: f32) -> Result<(), anyhow::Error>;
}

#[derive(Clone, Debug)]
pub struct RequestServicePub {
    private_ip: String
}

impl RequestServicePub {

    // pub fn new() -> Self {

    //     let local_ip = match local_ip() {
    //         Ok(ip) => {
                
    //         },
    //         Err(err) => {
    //             error!("{:?}", err);
    //             panic!("{:?}", err)
    //         }
    //     };
    // }

}

#[async_trait]
impl RequestService for RequestServicePub {

    async fn request_metric_to_elastic(cpu_usage: f32) -> Result<(), anyhow::Error> {
        
        let es_conn = get_elastic_conn();
        
        

        Ok(())
    }

}