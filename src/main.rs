use std::env;
use std::io::stdout;
use std::io::Write;
use std::process;
use std::str;
use std::time::Duration;

use clap::Parser;
use dotenvy::dotenv;
use eventsource_client as es;
use eventsource_client::Error;
use futures::TryStreamExt;
use futures::Stream;
use serde_json::{Result as JsonResult, Value};

use crate::args::{AnswerCommand, ClapArgs};
use crate::constants::{JSON_TYPE, KEY_MODEL, KEY_TOGETHER_API, MODELS_ENDPOINT, PROMPT_TOKEN};
use crate::embeddings::call_embeddings;
use crate::models::{call_list_models, find_model_config};
use crate::rest_client::create_client;

mod args;
mod models;
mod constants;
mod rest_client;
mod embeddings;

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
        args::TogetherAiSubcommand::Embeddings(embeddings) => {
            call_embeddings(embeddings).await.unwrap();
        }
    }
}

async fn call_streaming(args: AnswerCommand) -> Result<(), es::Error> {

    let model = env::var(KEY_MODEL).or::<String>(Ok("togethercomputer/llama-2-70b-chat".to_string())).unwrap();
    let mut models = vec![];
    let models_clone = args.models.clone();
    if args.all_models {
        let res = create_client(MODELS_ENDPOINT).await;
        match res {
            Ok(res) => {
                let sorted_models = models::list_chat_models(res).await;
                for model in sorted_models {
                    models.push(model);
                }
            }
            Err(err) => {
                eprintln!("Could not fetch all models: {}", err);
                process::exit(1);
            }
        }
    }
    if models.is_empty() {
        if models_clone.is_some() {
            models = models_clone.unwrap();
        } else {
            models.push(model.clone());
        }
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
    let user_prompt = read_model_prompt(&args);
    let model_config = find_model_config(model.clone()).await.unwrap();
    let model_prompt = model_config.prompt;
    let stop_words = model_config.stop_words;
    let prompt = str::replace(&model_prompt, PROMPT_TOKEN, user_prompt.as_str());

    let max_tokens = args.max_tokens.unwrap_or(512);
    let temperature = args.temperature.unwrap_or(0.0);
    let top_p = args.top_p.unwrap_or(0.7);
    let top_k = args.top_k.unwrap_or(50);
    let repetition_penalty = args.repetition_penalty.unwrap_or(1.);

    let body = format!("{{\
        \"model\": \"{model}\",\
        \"prompt\": \"{prompt}\",\
        \"max_tokens\": {max_tokens},\
        \"temperature\": {temperature},\
        \"top_p\": {top_p},\
        \"top_k\": {top_k},\
        \"repetition_penalty\": {repetition_penalty},\
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

    let mut stream = tail_events(client, stop_words.clone());

    while let Ok(Some(_)) = stream.try_next().await {}
    Ok(())
}

fn read_model_prompt(args: &AnswerCommand) -> String {
    let prompt = args.prompt.clone();
    if !args.prompt_is_file {
        return prompt
    } else {
        let res = std::fs::read_to_string(prompt);
        match res {
            Ok(res) => {
                return res;
            }
            Err(err) => {
                eprintln!("Could not read prompt file: {}", err);
                process::exit(1);
            }
        }
    }
}

fn tail_events(client: impl es::Client, stop_words: Vec<String>) -> impl Stream<Item = Result<(), ()>> {
    client
        .stream()
        .map_ok(move |event| match event {
            es::SSE::Event(ev) => {
                match extract_text(ev.data) {
                    Ok(token) => {
                        let mut is_stop_word = false;
                        for stop_word in &stop_words {
                            if token == *stop_word {
                                is_stop_word = true;
                                break; // break out of the for loop
                            }
                        }
                        if !is_stop_word {
                            // Only print if not a stop word
                            print!("{token}");
                            let _ = stdout().flush();
                        }
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

