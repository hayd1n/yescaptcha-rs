use serde::{Deserialize, Serialize};

use crate::{Task, TaskConfig, TaskSolution};

// API Information:
// https://yescaptcha.atlassian.net/wiki/spaces/YESCAPTCHA/pages/229796/NoCaptchaTaskProxyless+reCaptcha+V2

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaV2 {
    pub task_id: String,
}

impl Task for ReCaptchaV2 {
    type Solution = ReCaptchaV2Solution;

    fn task_id(&self) -> &str {
        &self.task_id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TaskType {
    NoCaptchaTaskProxyless,
    RecaptchaV2TaskProxyless,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaV2Config {
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub website_key: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub is_invisible: bool,
}

impl TaskConfig for ReCaptchaV2Config {
    type Task = ReCaptchaV2;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReCaptchaV2Solution {
    pub g_recaptcha_response: String,
}

impl TaskSolution for ReCaptchaV2Solution {}
