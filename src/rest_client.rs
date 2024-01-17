use std::env;
use reqwest::{Error, Response};
use crate::constants::KEY_TOGETHER_API;

pub(crate) async fn create_client(end_point: &str) -> Result<Response, Error> {
    let together_api_key = env::var(KEY_TOGETHER_API).unwrap();
    let client = reqwest::Client::new();
    let res = client
        .get(end_point)
        .header("Authorization", format!("Bearer {together_api_key}").as_str())
        .send()
        .await;
    res
}

pub(crate) async fn create_embedding(end_point: &str, data: String, model: String) -> Result<Response, Error> {
    let together_api_key = env::var(KEY_TOGETHER_API).unwrap();
    let client = reqwest::Client::new();
    let body = format!("{{\"input\": \"{}\", \"model\": \"{}\"}}", data, model);
    println!("{}", body);
    let res = client
        .post(end_point)
        .header("Authorization", format!("Bearer {together_api_key}").as_str())
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await;
    res
}