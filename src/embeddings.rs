use std::fs;
use std::path::Path;
use std::process;

use reqwest::Error;
use serde_json::Error as JsonError;
use serde_json::Value as JsonValue;

use crate::args::EmbeddingsCommand;
use crate::constants::{DEFAULT_EMBEDDING_MODEL, EMBEDDINGS_ENDPOINT};
use crate::rest_client::create_embedding;

pub(crate) async fn call_embeddings(embeddings: EmbeddingsCommand) -> Result<(), Error> {
    let file = embeddings.file;
    let folder = embeddings.folder;
    create_folder_if_not_exists(&folder);
    let models_option = embeddings.models;
    let models = if models_option.is_some() {
        models_option.unwrap()
    } else {
        vec![DEFAULT_EMBEDDING_MODEL.to_string()]
    };
    let file_content = std::fs::read_to_string(Path::new(&file));
    match file_content {
        Ok(content) => {
            for model in models {
                let res = create_embedding(EMBEDDINGS_ENDPOINT, content.clone(), model.clone()).await?;
                let text = res.text().await?;
                write_to_file(&folder, model, &text);
            }
        }
        Err(err) => {
            eprintln!("Could not read file: {}", err);
        }
    }
    Ok(())
}

fn write_to_file(folder: &String, model: String, text: &String) -> Result<(), JsonError> {
    let json = serde_json::from_str::<JsonValue>(&text)?;
    let data_option = json.get("data");
    match data_option {
        Some(data) => {
            let array = data.as_array().ok_or::<Vec<JsonValue>>(vec![]).unwrap();
            if array.len() == 0 {
                eprintln!("No embeddings found in JSON: {}", text);
                return Ok(())
            }
            let first = array.get(0).unwrap();
            let index = first["index"].as_i64().ok_or(-1).unwrap();
            if index == -1 {
                eprintln!("Could not find index in JSON: {}", text);
                return Ok(())
            }
            let file_name = format!("{}/{}_{}.json", folder, model.replace("/", "_"), index.to_string());
            let res = fs::write(file_name.clone(), text);
            if res.is_err() {
                eprintln!("Could not write file {}: {}", file_name, res.err().unwrap());
            } else {
                println!("Wrote file: {}", file_name);
            }
        }
        None => {
            eprintln!("No data found in JSON: {}", text);
            return Ok(())
        }
    }
    Ok(())
}

fn create_folder_if_not_exists(folder: &String) {
    let target_folder = Path::new(&folder);
    if !target_folder.exists() {
        println!("Creating folder: {}", folder);
        let res = fs::create_dir_all(target_folder);
        if res.is_err() {
            eprintln!("Could not create folder: {}. Good bye!", folder);
            process::exit(1);
        }
    }
}

