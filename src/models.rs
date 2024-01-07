use std::env;
use reqwest::{Error, Response};
use serde_json::Value;
use crate::constants::{KEY_TOGETHER_API, PROMPT_TOKEN};

pub(crate) async fn call_list_models() {
    let res = create_client().await;
    match res {
        Ok(res) => {
            let models = extract_models_array(res).await;
            let mut sorted_models = models.iter()
                .filter(|model| model["name"].as_str().is_some())
                .filter(|model| model["display_type"].as_str().is_some())
                .filter(|model| model["display_type"].as_str().unwrap() == "chat")
                .map(|model| {
                model["name"].as_str().unwrap()
            }).collect::<Vec<_>>().to_vec();
            sorted_models.sort();
            for model in sorted_models {
                println!("{}", model);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

async fn extract_models_array(res: Response) -> Vec<Value> {
    let body = res.text().await.unwrap();
    let v: Value = serde_json::from_str(&body).unwrap();
    let models = v.as_array().unwrap();
    models.clone()
}

pub(crate) async fn find_model_prompt(model_search: String) -> Result<String, Error> {
    let res = create_client().await;
    match res {
        Ok(res) => {
            let models = extract_models_array(res).await;
            let model = models.iter().find(|model| model["name"].as_str().unwrap() == model_search).unwrap();
            return match model["config"].as_object() {
                Some(config) => {
                    let prompt = config["prompt_format"].as_str().unwrap();
                    Ok(prompt.to_string())
                }
                None => {
                    eprintln!("Warning: Could not find prompt_format in model config.");
                    Ok(PROMPT_TOKEN.to_string())
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            Ok(PROMPT_TOKEN.to_string())
        }
    }
}

async fn create_client() -> Result<Response, Error> {
    let together_api_key = env::var(KEY_TOGETHER_API).unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.together.xyz/api/models?&info")
        .header("Authorization", format!("Bearer {together_api_key}").as_str())
        .send()
        .await;
    res
}
