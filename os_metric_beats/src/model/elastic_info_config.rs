use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct ElasticInfoConfig {
    pub hosts: Vec<String>,
    pub es_id: Option<String>,
    pub es_pw: Option<String>,
    pub index_pattern: String,
}
