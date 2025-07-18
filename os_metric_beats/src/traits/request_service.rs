use crate::common::*;

use crate::model::metric_info::*;

#[async_trait]
pub trait RequestService {
    async fn request_metric_to_elastic(
        &self,
        index_name: String,
        metric_info: MetricInfo,
    ) -> Result<(), anyhow::Error>;
}
