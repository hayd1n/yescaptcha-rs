use std::time::Duration;
use tokio::time::sleep;
use yescaptcha::{
    task::recaptcha_v2::{ReCaptchaV2Config, TaskType},
    Client, TaskResult,
};

#[tokio::main]
async fn main() {
    let client_key = match std::env::var("YES_CAPTCHA_CLIENT_KEY") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Please set YES_CAPTCHA_CLIENT_KEY environment variable");
            return;
        }
    };

    // Create a new YesCaptcha client
    let client = Client::new(&client_key);

    // Create a new ReCaptchaV2 task
    let task_config = ReCaptchaV2Config {
        website_url: "https://www.google.com/recaptcha/api2/demo".to_string(),
        website_key: "6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-".to_string(),
        task_type: TaskType::NoCaptchaTaskProxyless,
        is_invisible: false,
    };

    // Send the task to the YesCaptcha API
    let task = client.create_task(task_config).await.unwrap();

    // Wait for the task to be completed
    loop {
        // Get the task result
        let result = client.get_task_result(&task).await.unwrap();

        match result {
            TaskResult::Processing => {
                println!("Task is not completed yet");
            }
            TaskResult::Ready(solution) => {
                println!("Solution: {:#?}", solution);
                // Exit the loop once the task is completed
                break;
            }
        }

        // Wait for 5 seconds before checking again
        sleep(Duration::from_secs(5)).await;
    }
}
