use std::time::Duration;

use eventsource_client as es;
use futures::Stream;
use serde_json::{Result as JsonResult, Value};
use std::io::Write;
use std::io::stdout;
use dotenvy::dotenv;
use std::env;
use std::str;
use futures::{TryStreamExt};
use crate::args::{AnswerCommand, ClapArgs};
use clap::Parser;
use eventsource_client::Error;
use crate::constants::{JSON_TYPE, KEY_MODEL, KEY_TOGETHER_API, PROMPT_TOKEN};
use crate::models::{call_list_models, find_model_prompt};

mod args;
mod models;
mod constants;

#[tokio::main]
async fn main() {
    let dotenv_res= dotenv();
    let args = ClapArgs::parse();
    if dotenv_res.is_err() && env::var(KEY_TOGETHER_API).is_err() {
        eprintln!("Error: Could not find .env file or TOGETHER_API_KEY environment variable.");
        return;
    }
    match args.subcommand {
        args::TogetherAiSubcommand::ListModels(_) => {
            call_list_models().await;
        }
        args::TogetherAiSubcommand::Answer(answer) => {
            call_streaming(answer).await.unwrap();
        }
    }
}

async fn call_streaming(args: AnswerCommand) -> Result<(), es::Error> {

    let model = env::var(KEY_MODEL).or::<String>(Ok("togethercomputer/llama-2-70b-chat".to_string())).unwrap();
    let mut models = vec![];
    let models_clone = args.models.clone();
    if models_clone.is_some() {
        models = models_clone.unwrap();
    } else {
        models.push(model.clone());
    }
    let print_model = models.len() > 1;
    for model in models {
        if print_model {
            println!("");
            println!("# Model: {}", model);
        }
        process_single_model(&args, model).await?;
    }
    Ok(())
}

async fn process_single_model(args: &AnswerCommand, model: String) -> Result<(), Error> {
    let together_api_key = env::var(KEY_TOGETHER_API).unwrap();
    let user_prompt = args.prompt.clone();
    let model_prompt = find_model_prompt(model.clone()).await.unwrap();
    let prompt = str::replace(&model_prompt, PROMPT_TOKEN, user_prompt.as_str());

    let temperature = 0.0;
    let body = format!("{{\
        \"model\": \"{model}\",\
        \"prompt\": \"{prompt}\",\
        \"max_tokens\": 512,\
        \"temperature\": {temperature},\
        \"top_p\": 0.7,\
        \"top_k\": 50,\
        \"repetition_penalty\": 1,\
        \"stream_tokens\": true}}");
    let client = es::ClientBuilder::for_url("https://api.together.xyz/inference")?
        .header("Authorization", format!("Bearer {together_api_key}").as_str())?
        .header("accept", JSON_TYPE)?
        .header("content-type", JSON_TYPE)?
        .body(body)
        .method("POST".to_string())
        .reconnect(
            es::ReconnectOptions::reconnect(true)
                .retry_initial(false)
                .delay(Duration::from_secs(1))
                .backoff_factor(2)
                .delay_max(Duration::from_secs(60))
                .build(),
        )
        .build();

    let mut stream = tail_events(client);

    while let Ok(Some(_)) = stream.try_next().await {}
    Ok(())
}

fn tail_events(client: impl es::Client) -> impl Stream<Item = Result<(), ()>> {
    client
        .stream()
        .map_ok(|event| match event {
            es::SSE::Event(ev) => {
                match extract_text(ev.data) {
                    Ok(token) => {
                        print!("{token}");
                        let _ = stdout().flush();
                    }
                    Err(_) => {
                        print!("")
                    }
                }

            }
            es::SSE::Comment(comment) => {
                let token = extract_text(comment).unwrap();
                print!("{token}")
            }
        })
        .map_err(|err| {
            if err.to_string() != "eof" {
                eprintln!("{}", err.to_string());
                eprintln!("error streaming events: {:?}", err)
            }
        })
}

fn extract_text(data: String) -> JsonResult<String> {
    let v: Value = serde_json::from_str(&data)?;
    match v["choices"].as_array() {
        Some(choices) => {
            let mut text = String::new();
            for choice in choices {
                match choice["text"].as_str() {
                    Some(t) => text.push_str(t),
                    None => text.push_str(""),
                }
            }
            Ok(text)
        }
        None => Ok("".to_string()),
    }
}

