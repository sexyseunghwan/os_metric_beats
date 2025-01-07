use crate::common::*;

use crate::repository::es_repository::*;

use crate::model::metric_info::*;

#[async_trait]
pub trait RequestService {
    async fn request_metric_to_elastic(
        &self,
        index_name: String,
        metric_info: MetricInfo,
    ) -> Result<(), anyhow::Error>;
}

#[derive(Clone, Debug, new)]
pub struct RequestServicePub;

#[async_trait]
impl RequestService for RequestServicePub {
    async fn request_metric_to_elastic(
        &self,
        index_name: String,
        metric_info: MetricInfo,
    ) -> Result<(), anyhow::Error> {
        let es_conn = get_elastic_conn();
        //let index_pattern = es_conn.index_pattern();
        let document: Value = serde_json::to_value(&metric_info)?;

        es_conn.post_doc(&index_name, document).await?;

        Ok(())
    }
}
