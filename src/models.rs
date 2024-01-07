use std::env;

use reqwest::{Error, Response};
use serde_json::{Map, Value};

use crate::constants::{KEY_TOGETHER_API, PROMPT_TOKEN};

pub(crate) struct ModelConfig {
    pub(crate) prompt: String,
    pub(crate) stop_words: Vec<String>,
}

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

pub(crate) async fn find_model_config(model_search: String) -> Result<ModelConfig, Error> {
    find_in_model(model_search, |config| {
        let prompt = config["prompt_format"].as_str().unwrap();
        let stop_words_vec = config["stop"].as_array().unwrap();
        let stop_words = stop_words_vec.iter().map(|stop_word| stop_word.as_str().unwrap().to_string()).collect();
        ModelConfig {
            prompt: prompt.to_string(),
            stop_words,
        }
    }, || ModelConfig {
        prompt: PROMPT_TOKEN.to_string(),
        stop_words: vec![],
    }).await
}

pub(crate) async fn find_in_model<T>(model_search: String,
                                     extract_fn: fn(&Map<String, Value>) -> T,
                                     default_fn: fn() -> T) -> Result<T, Error> {
    let res = create_client().await;
    match res {
        Ok(res) => {
            let models = extract_models_array(res).await;
            let model = models.iter().find(|model| model["name"].as_str().unwrap() == model_search)
                .expect("Could not find model. Make sure this model exists and is a chat model.");
            let config_object: Option<&Map<String, Value>> = model["config"].as_object();
            return match config_object {
                Some(config) => {
                    Ok(extract_fn(config))
                }
                None => {
                    eprintln!("Warning: Could not find prompt_format in model config.");
                    Ok(default_fn())
                }
            };
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            Ok(default_fn())
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
