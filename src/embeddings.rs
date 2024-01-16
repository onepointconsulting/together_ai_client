use std::path::Path;
use crate::args::EmbeddingsCommand;
use crate::constants::{DEFAULT_EMBEDDING_MODEL, EMBEDDINGS_ENDPOINT};
use crate::rest_client::{create_client, create_embedding};
use serde_json::{Map, Value as JsonValue};

pub(crate) async fn call_embeddings(embeddings: EmbeddingsCommand) {
    let file = embeddings.file;
    let folder = embeddings.folder;
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
                let res = create_embedding(EMBEDDINGS_ENDPOINT, content.clone(), model).await;
                println!("{:?}", res);
                match res {
                    Ok(res) => {
                        let body = res.text().await.unwrap();
                        let result = serde_json::from_str::<JsonValue>(&body);
                        match result {
                            Ok(v) => {
                                let obj_option = v.as_object();
                                match obj_option {
                                    Some(obj) => {
                                        let data = obj.get("data");
                                        println!("{}", data.unwrap());
                                        match data {
                                            Some(data) => {
                                                let embeddings = data.as_array().unwrap();
                                                for embedding in embeddings {
                                                    let embedding_obj = embedding.as_object().unwrap();
                                                    let embedding_array = embedding_obj.get("embedding").unwrap().as_array();
                                                    match embedding_array {
                                                        Some(embedding_array) => {
                                                            println!("{}", embedding_array.len());
                                                        }
                                                        None => { }
                                                    }
                                                }
                                            }
                                            None => {
                                                eprintln!("Could not find data in response");
                                            }
                                        }
                                    }
                                    None => {
                                        eprintln!("Could not find object in response");
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("Could not parse response: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Could not fetch all models: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Could not read file: {}", err);
        }
    }
}

