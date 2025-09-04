use actix_web::rt;
use bytes::Bytes;
use futures::{channel::mpsc, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use futures_util::StreamExt;
use base64::Engine;

/// Pulls a model from Ollama by making a POST request to the Ollama server.
/// Returns Ok(()) if the pull was successful, or an error otherwise.
pub async fn pull_ollama_model(model_name: &str, ollama_host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/pull", ollama_host);
    let body = json!({ "model": model_name });

    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        tracing::info!("Pulled model from Ollama: {:?}", model_name);
        Ok(())
    } else {
        let status = resp.status();
        let message = resp.text().await.unwrap();
        let message = format!("Ollama error: {status}: {message}");
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message)))
    }
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
) -> Result<mpsc::Receiver<Result<Bytes, reqwest::Error>>, reqwest::Error> {
    tracing::info!("Sending images to Ollama: {:?}", images_batch.len());
    let images_b64: Vec<String> = images_batch
        .iter()
        .map(|img| base64::engine::general_purpose::STANDARD.encode(img))
        .collect();

    let body = json!({
        "prompt": prompt,
        "images": images_b64,
        "model": model.clone(),
        "stream": true,
    });

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
