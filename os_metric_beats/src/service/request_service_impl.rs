use crate::common::*;

use crate::repository::es_repository::*;

use crate::model::metric_info::*;

use crate::traits::request_service::*;

#[derive(Clone, Debug, new)]
pub struct RequestServiceImpl;

#[async_trait]
impl RequestService for RequestServiceImpl {
    async fn request_metric_to_elastic(
        &self,
        index_name: String,
        metric_info: MetricInfo,
    ) -> Result<(), anyhow::Error> {
        let es_conn: Arc<EsRepositoryPub> = get_elastic_conn();
        let document: Value = serde_json::to_value(&metric_info)?;

        es_conn.post_doc(&index_name, document).await?;

        Ok(())
    }
}
