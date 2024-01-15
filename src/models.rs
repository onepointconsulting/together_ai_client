use std::env;

use reqwest::{Error, Response};
use serde_json::{Map, Value as JsonValue};

use crate::constants::{KEY_TOGETHER_API, PROMPT_TOKEN};

pub(crate) struct ModelConfig {
    pub(crate) prompt: String,
    pub(crate) stop_words: Vec<String>,
}

pub(crate) async fn call_list_models() {
    let res = create_client().await;
    match res {
        Ok(res) => {
            let sorted_models = list_chat_models(res).await;
            for model in sorted_models {
                println!("{}", model);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}

pub(crate) async fn list_chat_models(res: Response) -> Vec<String> {
    let models = extract_models_array(res).await;
    let mut sorted_models = models.iter()
        .filter(|model| model["name"].as_str().is_some())
        .filter(|model| model["display_type"].as_str().is_some())
        .filter(|model| model["display_type"].as_str().unwrap() == "chat")
        .map(|model| {
            model["name"].as_str().unwrap().to_string()
        }).collect::<Vec<_>>().to_vec();
    sorted_models.sort();
    sorted_models
}

pub(crate) async fn extract_models_array(res: Response) -> Vec<JsonValue> {
    let body = res.text().await.unwrap();
    let result = serde_json::from_str::<JsonValue>(&body);
    match result {
        Ok(v) => {
            let models = v.as_array().unwrap();
            models.clone()
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            vec![]
        }
    }
}

pub(crate) async fn find_model_config(model_search: String) -> Result<ModelConfig, Error> {
    find_in_model(model_search, |config| {
        let contains_prompt = config.contains_key("prompt_format");
        let prompt = if contains_prompt  { config["prompt_format"].as_str().or(Some(PROMPT_TOKEN)).unwrap() }
            else { PROMPT_TOKEN };
        let empty = vec![];
        let contains_stop = config.contains_key("stop");
        let stop_words_vec = if contains_stop { config["stop"].as_array().or(Some(&empty)).unwrap() }
            else { &empty };
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
                                     extract_fn: fn(&Map<String, JsonValue>) -> T,
                                     default_fn: fn() -> T) -> Result<T, Error> {
    let res = create_client().await;
    match res {
        Ok(res) => {
            let models = extract_models_array(res).await;
            let model_option = models.iter().find(|model| model["name"].as_str().unwrap() == model_search);
            match model_option {
                Some(model) => {
                    let config_object: Option<&Map<String, JsonValue>> = model["config"].as_object();
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
                None => {
                    eprintln!("Warning: Could not find model {}.", model_search);
                    Ok(default_fn())
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            Ok(default_fn())
        }
    }
}


pub(crate) async fn create_client() -> Result<Response, Error> {
    let together_api_key = env::var(KEY_TOGETHER_API).unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.together.xyz/api/models?&info")
        .header("Authorization", format!("Bearer {together_api_key}").as_str())
        .send()
        .await;
    res
}
