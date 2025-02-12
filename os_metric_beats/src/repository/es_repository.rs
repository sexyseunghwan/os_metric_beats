use crate::common::*;

use crate::utils_module::io_utils::*;

use crate::model::elastic_info_config::*;

use crate::env_configuration::env_config::*;

#[doc = "Elasticsearch connection 을 싱글톤으로 관리하기 위한 전역 변수."]
static ELASTICSEARCH_CLIENT: once_lazy<Arc<EsRepositoryPub>> =
    once_lazy::new(|| initialize_elastic_clients());

#[doc = "Function to initialize Elasticsearch connection instances"]
pub fn initialize_elastic_clients() -> Arc<EsRepositoryPub> {
    let cluster_config: ElasticInfoConfig =
        match read_toml_from_file::<ElasticInfoConfig>(&ELASTIC_SERVER_INFO) {
            Ok(cluster_config) => cluster_config,
            Err(e) => {
                error!("{:?}", e);
                panic!("{:?}", e)
            }
        };

    let es_hosts: Vec<String> = cluster_config.hosts().clone();
    let es_id: String = cluster_config.es_id().clone().unwrap_or(String::from(""));
    let es_pw: String = cluster_config.es_pw().clone().unwrap_or(String::from(""));
    let index_pattern: String = cluster_config.index_pattern().clone();

    let es_helper: EsRepositoryPub =
        match EsRepositoryPub::new(es_hosts, &es_id, &es_pw, &index_pattern) {
            Ok(es_helper) => es_helper,
            Err(err) => {
                error!("{:?}", err);
                panic!("{:?}", err)
            }
        };

    Arc::new(es_helper)
}

#[doc = "엘라스틱 서치 커넥션을 가져와주는 get() 함수"]
pub fn get_elastic_conn() -> Arc<EsRepositoryPub> {
    let es_conn = &ELASTICSEARCH_CLIENT;
    Arc::clone(es_conn)
}

#[async_trait]
pub trait EsRepository {
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
    index_pattern: String,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryPub {
    pub fn new(
        hosts: Vec<String>,
        es_id: &str,
        es_pw: &str,
        index_pattern: &str,
    ) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<EsClient> = Vec::new();

        for url in hosts {
            let parse_url: String;

            /* Elasticsearch 에 비밀번호를 설정해둔 경우와 그렇지 않은 경우를 고려함 */
            if es_id != "" && es_pw != "" {
                parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            } else {
                parse_url = format!("http://{}", url);
            }

            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: elasticsearch::http::transport::Transport =
                TransportBuilder::new(conn_pool)
                    .timeout(Duration::new(5, 0))
                    .build()?;

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: EsClient = EsClient::new(url, elastic_conn);
            es_clients.push(es_client);
        }

        Ok(EsRepositoryPub {
            es_clients,
            index_pattern: index_pattern.to_string(),
        })
    }

    #[doc = "Common logic: common node failure handling and node selection"]
    /// # Arguments
    /// * `operation` - Elasticsearch 특정 노드의 함수
    ///
    /// # Returns
    /// * Result<T, anyhow::Error>
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error: Option<anyhow::Error> = None;
        let mut rng: StdRng = StdRng::from_entropy(); /* 랜덤 시드로 생성 */

        /*  클라이언트 목록을 셔플 -> StdRng를 사용하여 셔플 */
        let mut shuffled_clients: Vec<EsClient> = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);

        /* 셔플된 클라이언트들에 대해 순차적으로 operation 수행 */
        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        /* 모든 노드에서 실패했을 경우 에러 반환 */
        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}

#[async_trait]
impl EsRepository for EsRepositoryPub {
    #[doc = "Elasticsearch 에 색인해주는 함수"]
    /// # Arguments
    /// * `index_name`- 인덱스 이름
    /// * `document`  - 색인할 문서
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error> {
        /* 클로저 내에서 사용할 복사본을 생성 */
        let document_clone: Value = document.clone();

        let response: Response = self
            .execute_on_any_node(|es_client| {
                /* 클로저 내부에서 클론한 값 사용 */
                let value = document_clone.clone();
                async move {
                    let response = es_client
                        .es_conn
                        .index(IndexParts::Index(index_name))
                        .body(value)
                        .send()
                        .await?;

                    Ok(response)
                }
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message: String = format!(
                "[Elasticsearch Error][post_doc()] Failed to index document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }
}
