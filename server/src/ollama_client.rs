use actix_web::rt;
use bytes::Bytes;
use futures::{channel::mpsc, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures_util::StreamExt;
use base64::Engine;

#[derive(Deserialize)]
struct OllamaPullResponse {
    status: String
}

#[derive(Deserialize)]
struct OllamaModel {
    model: String,
}

#[derive(Deserialize)]
struct OllamaListModelsResponse {
    models: Vec<OllamaModel>
}

/// Pulls a model from Ollama by making a POST request to the Ollama server.
/// Returns Ok(()) if the pull was successful, or an error otherwise.
pub async fn pull_ollama_model(model_name: &str, ollama_host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Check if the model exists before pulling
    let check_url = format!("{}/api/tags", ollama_host);
    let check_resp = client
        .get(&check_url)
        .send()
        .await?;

    if check_resp.status().is_success() {
        let resp_json: OllamaListModelsResponse = check_resp.json().await?;
        if resp_json.models.iter().any(|model| model.model == model_name) {
            tracing::info!("Model '{}' already exists on Ollama, skipping pull.", model_name);
            return Ok(());
        }
    }

    let url = format!("{}/api/pull", ollama_host);
    let body = json!({ "model": model_name, "stream": true });
    tracing::info!("Pulling model {} from Ollama: {:?}", model_name, url);

    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await?;

    let mut stream = resp.bytes_stream();

    tracing::info!("Received response from Ollama pull");
    while let Some(chunk) = stream.next().await {
        if let Ok(chunk) = chunk {
            let response: OllamaPullResponse = serde_json::from_slice(&chunk).unwrap();
            if response.status == "success" {
                break;
            }
            tracing::info!("Received response from Ollama pull: {:?}", response.status);
        } else {
            let err = chunk.err().unwrap();
            tracing::error!("Error sending chunk to channel: {:?}", err);
            return Err(err.into());
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize)]
pub struct OllamaResponse {
    model: String,
    created_at: String,
    pub response: String,
    pub done: bool,
}

pub async fn send_to_ollama(
    images_batch: Vec<Vec<u8>>,
    prompt: String,
    model: String,
    ollama_host: String,
    num_predict: Option<i32>,
) -> Result<mpsc::Receiver<Result<Bytes, reqwest::Error>>, reqwest::Error> {
    tracing::info!("Sending images to Ollama: {:?}", images_batch.len());
    let images_b64: Vec<String> = images_batch
        .iter()
        .map(|img| base64::engine::general_purpose::STANDARD.encode(img))
        .collect();

    let mut body = json!({
        "prompt": prompt,
        "images": images_b64,
        "model": model.clone(),
        "stream": true,
    });

    if let Some(num_predict_val) = num_predict {
        body["options"] = json!({
            "num_predict": num_predict_val
        });
    }

    let url = format!("{}/api/generate", ollama_host);

    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await?;
  
    let mut stream = resp.bytes_stream();
    let (mut tx, rx) = mpsc::channel(100);

    rt::spawn(async move {
        tracing::info!("Received response from Ollama");
        while let Some(chunk) = stream.next().await {
            if let Ok(chunk) = chunk {
                if let Err(e) = tx.send(Ok(chunk)).await {
                    tracing::error!("Error sending chunk to channel: {:?}", e);
                    break;
                }
            } else {
                if let Err(e) = tx.send(Err(chunk.err().unwrap())).await {
                    tracing::error!("Error sending chunk to channel: {:?}", e);
                    break;
                }
                break;
            }
        }
        tx.close().await.unwrap();
    });
    Ok(rx)
}
