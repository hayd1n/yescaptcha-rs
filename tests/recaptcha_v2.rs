use yescaptcha::task::recaptcha_v2::{ReCaptchaV2Config, TaskType};

#[tokio::test]
async fn recaptcha_v2() {
    let api_url = std::env::var("YES_CAPTCHA_API_URL");

    let client_key = match std::env::var("YES_CAPTCHA_CLIENT_KEY") {
        Ok(val) => val,
        Err(_) => {
            panic!("Please set YES_CAPTCHA_CLIENT_KEY environment variable");
        }
    };

    let mut builder = yescaptcha::ClientBuilder::new().client_key(client_key);
    if let Ok(api_url) = api_url {
        builder = builder.api_url(api_url.parse().unwrap());
    }

    let client = builder.build().unwrap();

    let task_config = ReCaptchaV2Config {
        website_url: "https://www.google.com/recaptcha/api2/demo".to_string(),
        website_key: "6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-".to_string(),
        task_type: TaskType::NoCaptchaTaskProxyless,
        is_invisible: false,
    };

    let task = client.create_task(task_config).await.unwrap();

    loop {
        let result = client.get_task_result(&task).await.unwrap();

        match result {
            yescaptcha::TaskResult::Processing => {
                println!("Task is not completed yet");
            }
            yescaptcha::TaskResult::Ready(solution) => {
                println!("Solution: {:#?}", solution);
                break;
            }
        }

        // Wait for 5 seconds before checking again
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
