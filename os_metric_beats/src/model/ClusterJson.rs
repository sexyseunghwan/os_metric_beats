use crate::common::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterJson {
    pub hosts: Vec<String>,
    pub es_id: String,
    pub es_pw: String,
}