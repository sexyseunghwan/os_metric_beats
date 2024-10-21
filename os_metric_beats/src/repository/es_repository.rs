use crate::common::*;

use crate::utils_module::io_utils::*;

use crate::model::ClusterJson::*;

/*
    Elasticsearch connection 을 싱글톤으로 관리하기 위한 전역 변수.
*/
static ELASTICSEARCH_CLIENT: once_lazy<Arc<EsRepositoryPub>> = once_lazy::new(|| {
    initialize_elastic_clients()
});


/*
    Function to initialize Elasticsearch connection instances
*/
pub fn initialize_elastic_clients() -> Arc<EsRepositoryPub> {
    
    let cluster_config: ClusterJson = match read_json_from_file::<ClusterJson>("./datas/server_info.json") {
        Ok(cluster_config) => cluster_config,
        Err(err) => {
            error!("{:?}", err);
            panic!("{:?}", err)
        }
    };
    
    let es_helper = match EsRepositoryPub::new(
            cluster_config.hosts.clone(), 
            &cluster_config.es_id, 
            &cluster_config.es_pw) {
                Ok(es_helper) => es_helper,
                Err(err) => {
                    error!("{:?}", err);
                    panic!("{:?}", err)
                }
            };
    
    Arc::new(es_helper)
}


/*
    엘라스틱 서치 커넥션을 가져와주는 get() 함수
*/
pub fn get_elastic_conn() -> Arc<EsRepositoryPub> {
    let es_conn = &ELASTICSEARCH_CLIENT;
    Arc::clone(&es_conn)
}

#[async_trait]
pub trait EsRepository {
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch
}


impl EsRepositoryPub {
    
    pub fn new(hosts: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {

        let mut es_clients: Vec<EsClient> = Vec::new();
        
        for url in hosts {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            let elastic_conn = Elasticsearch::new(transport);
            let es_client = EsClient::new(url, elastic_conn);
            es_clients.push(es_client);
        }
        
        Ok(EsRepositoryPub{es_clients})
    }
    
    
    // Common logic: common node failure handling and node selection
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;
    
        // StdRng를 사용하여 Send 트레잇 문제 해결
        let mut rng = StdRng::from_entropy(); // 랜덤 시드로 생성
        
        // 클라이언트 목록을 셔플
        let mut shuffled_clients: Vec<EsClient> = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng); // StdRng를 사용하여 셔플
        
        // 셔플된 클라이언트들에 대해 순차적으로 operation 수행
        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }
        
        // 모든 노드에서 실패했을 경우 에러 반환
        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }

}


#[async_trait]
impl EsRepository for EsRepositoryPub {
    

    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error> {

        // 클로저 내에서 사용할 복사본을 생성
        let document_clone = document.clone();
        
        let response = self.execute_on_any_node(|es_client| {
            // 클로저 내부에서 클론한 값 사용
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
            let error_message = format!("[Elasticsearch Error][post_doc()] Failed to index document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }
}