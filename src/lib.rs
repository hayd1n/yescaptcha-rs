use std::{
    fmt::{self, Debug},
    time::Duration,
};

use reqwest::IntoUrl;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use url::Url;

pub mod task;

pub const API_URL_INTERNATIONAL: &str = "https://api.yescaptcha.com";
pub const API_URL_CHINA: &str = "https://cn.yescaptcha.com";

pub const DEFAULT_API_URL: &str = API_URL_INTERNATIONAL;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseBase {
    pub error_id: i32,
    pub error_code: Option<String>,
    pub error_description: Option<String>,
}

pub trait Task: serde::Serialize + DeserializeOwned {
    type Solution: TaskSolution + DeserializeOwned;

    fn task_id(&self) -> &str;
}

pub trait TaskConfig: serde::Serialize + DeserializeOwned {
    type Task: Task + DeserializeOwned;
}

pub trait TaskSolution: serde::Serialize + DeserializeOwned {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    Processing,
    Ready,
}

pub enum TaskResult<T>
where
    T: Task,
{
    Processing,
    Ready(T::Solution),
}

pub struct ClientBuilder {
    pub reqwest_client: reqwest::Client,
    pub api_url: Url,
    pub client_key: Option<String>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            reqwest_client: default_reqwest_builder().build().unwrap(),
            api_url: DEFAULT_API_URL.parse().unwrap(),
            client_key: None,
        }
    }

    pub fn api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn client_key(mut self, client_key: String) -> Self {
        self.client_key = Some(client_key);
        self
    }

    pub fn build(self) -> Result<Client, BuildError> {
        if self.client_key.is_none() {
            return Err(BuildError::InputError("client_key is required".to_string()));
        }

        Ok(Client {
            http_client: self.reqwest_client,
            api_url: self.api_url,
            client_key: self.client_key.unwrap(),
        })
    }
}

pub struct Client {
    pub http_client: reqwest::Client,
    pub api_url: Url,
    pub client_key: String,
}

impl Client {
    pub fn new(client_key: &str) -> Self {
        ClientBuilder::new()
            .client_key(client_key.to_string())
            .build()
            .unwrap()
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    async fn post<B: serde::Serialize, R: DeserializeOwned, U: IntoUrl>(
        &self,
        url: U,
        body: B,
    ) -> Result<R, ClientError> {
        let body = serde_json::to_string(&body).map_err(|_| {
            ClientError::RequestError("Failed to serialize request body".to_string())
        })?;

        let res = self
            .http_client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| ClientError::RequestError(format!("Failed to send request: {}", e)))?;

        let result = res
            .json::<R>()
            .await
            .map_err(|e| ClientError::ParseError(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }

    pub async fn create_task<T>(&self, task_config: T) -> Result<T::Task, ClientError>
    where
        T: TaskConfig,
    {
        let body = json!({
            "clientKey": self.client_key.clone(),
            "task": task_config
        });

        #[derive(Debug, Serialize, Deserialize)]
        struct CreateTaskResponse<T: TaskConfig> {
            #[serde(flatten)]
            response_base: ResponseBase,
            #[serde(flatten)]
            task: T::Task,
        }

        let response: CreateTaskResponse<T> = self
            .post(self.api_url.join("createTask").unwrap(), body)
            .await?;

        // Check if there was an error
        if response.response_base.error_id != 0 {
            return Err(ClientError::ApiError(response.response_base));
        }

        Ok(response.task)
    }

    pub async fn get_task_result<T>(&self, task: &T) -> Result<TaskResult<T>, ClientError>
    where
        T: Task,
    {
        let body = json!({
            "clientKey": self.client_key.clone(),
            "taskId": task.task_id()
        });

        #[derive(Debug, Serialize, Deserialize)]
        struct GetTaskResultResponse<T: Task> {
            #[serde(flatten)]
            response_base: ResponseBase,
            status: TaskStatus,
            solution: Option<T::Solution>,
        }

        let response: GetTaskResultResponse<T> = self
            .post(self.api_url.join("getTaskResult").unwrap(), body)
            .await?;

        // Check if there was an error
        if response.response_base.error_id != 0 {
            return Err(ClientError::ApiError(response.response_base));
        }

        match response.status {
            TaskStatus::Processing => Ok(TaskResult::Processing),
            TaskStatus::Ready => {
                // Check if there is a solution
                let solution = match response.solution {
                    Some(solution) => solution,
                    None => {
                        return Err(ClientError::ParseError("No solution provided".to_string()))
                    }
                };

                Ok(TaskResult::Ready(solution))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BuildError {
    InputError(String),
}

impl std::error::Error for BuildError {}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuildError::InputError(msg) => write!(f, "Input Error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClientError {
    InputError(String),
    RequestError(String),
    ParseError(String),
    ApiError(ResponseBase),
}

impl std::error::Error for ClientError {}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::InputError(msg) => write!(f, "Input Error: {}", msg),
            ClientError::RequestError(msg) => write!(f, "Request Error: {}", msg),
            ClientError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            ClientError::ApiError(response) => write!(f, "API Error: {:?}", response),
        }
    }
}

pub fn default_reqwest_builder() -> reqwest::ClientBuilder {
    reqwest::ClientBuilder::new().timeout(DEFAULT_TIMEOUT)
}
