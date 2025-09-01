use actix_web::{rt, web, Error, HttpRequest, HttpResponse};
use actix_ws::{AggregatedMessage, Message};
use futures_util::StreamExt as _;
use serde::Deserialize;
use serde_json::json;
use base64::Engine;

const IMAGE_BATCH_SIZE: usize = 5;

/// Pulls a model from Ollama by making a POST request to the Ollama server.
/// Returns Ok(()) if the pull was successful, or an error otherwise.
async fn pull_ollama_model(model_name: &str, ollama_host: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/pull", ollama_host);
    let body = json!({ "model": model_name });

    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let message = resp.text().await.unwrap();
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, message)))
    }
}


async fn handle_binary(
    images: &mut Vec<Vec<u8>>,
    bin: bytes::Bytes,
    session: &mut actix_ws::Session,
    last_prompt: &mut Option<String>,
    model: String,
    ollama_host: String,
) {
    images.push(bin.to_vec());

    if images.len() >= IMAGE_BATCH_SIZE {
        tracing::info!("Sending images to Ollama: {:?}", images.len());
        if let Some(prompt) = last_prompt.clone() {
            let images_batch = std::mem::take(images);
            send_to_ollama(images_batch, prompt, session, model, ollama_host).await;
        } else {
            let _ = session.text("No prompt received for image batch".to_string()).await;
        }
    }
}

async fn handle_text(
    session: &mut actix_ws::Session,
    images: &mut Vec<Vec<u8>>,
    text: String,
    last_prompt: &mut Option<String>,
    model: String,
    ollama_host: String,
) {
    *last_prompt = Some(text.clone());

    // If prompt updated, send the images to Ollama
    if !images.is_empty() {
        let images_batch = std::mem::take(images);
        send_to_ollama(images_batch, text, session, model, ollama_host).await;
    }
}

async fn handle_ping(session: &mut actix_ws::Session, msg: bytes::Bytes) {
    let _ = session.pong(&msg).await;
}

#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
}

async fn send_to_ollama(
    images_batch: Vec<Vec<u8>>,
    prompt: String,
    session: &mut actix_ws::Session,
    model: String,
    ollama_host: String,
) {
    let mut session_addr = session.clone();
    rt::spawn(async move {
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

        match reqwest::Client::new()
            .post(&url)
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                tracing::info!("Received response from Ollama");
                let mut first = true;
                let mut stream = resp.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    if let Ok(chunk) = chunk {
                        let response: OllamaResponse = serde_json::from_slice(&chunk).unwrap();
                        if response.done {
                            tracing::info!("Done");
                            let _ = session_addr.text(response.response).await;
                            break;
                        } else if first {
                            tracing::info!("First");
                            let _ = session_addr.text(format!("> {}", response.response)).await;
                            first = false;
                        } else {
                            tracing::info!("Not first");
                            let _ = session_addr.text(response.response).await;
                        }
                    }
                }
            }
            Err(e) => {
                let _ = session_addr.text(format!("Ollama error: {e}")).await;
            }
        };
    });
}

pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let model = std::env::var("VLM_MODEL").unwrap_or_else(|_| "llama3:latest".to_string());
    let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    if let Err(e) = pull_ollama_model(&model, &ollama_host).await {
        return Ok(HttpResponse::InternalServerError().body(e.to_string()));
    }

    let mut stream = stream.max_frame_size(1024*1024);

    rt::spawn(async move {
        let mut images: Vec<Vec<u8>> = Vec::new();
        let mut last_prompt: Option<String> = None;

        while let Some(msg) = stream.next().await {
            match msg {
                Ok(Message::Binary(bin)) => {
                    handle_binary(&mut images, bin, &mut session, &mut last_prompt, model.clone(), ollama_host.clone()).await;
                    tracing::info!("Received binary message");
                }
                Ok(Message::Text(text)) => {
                    handle_text(&mut session, &mut images, text.to_string(), &mut last_prompt, model.clone(), ollama_host.clone()).await;
                    tracing::info!("Received text message");
                }
                Ok(Message::Ping(msg)) => {
                    handle_ping(&mut session, msg).await;
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::error!("Error: {:?}", e);
                }
                res => {
                    match res {
                        Ok(msg) => {
                            tracing::info!("Received unknown message: {:?}", msg);
                        }
                        Err(e) => {
                            tracing::error!("Error: {:?}", e);
                        }
                    }
                }
            }
        }

        tracing::info!("Stream closed");
    });

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web::PayloadConfig};
    use actix_web_actors::ws;
    use awc::ws::Message;
    use bytes::Bytes;

    #[test]
    async fn test_send_two_large_binary_messages() {
        use actix_web::web;
        use actix_web::App;
        use futures_util::{SinkExt, StreamExt};

        // Start test server with ws_index route
        let mut srv = actix_test::start(|| {
            App::new().service(web::resource("/").to(ws_index).app_data(web::Data::new(PayloadConfig::new(2_usize.pow(20)))))
        });

        let mut framed = srv.ws().await.unwrap();

        // Prepare two large binary messages (> 300 KB)
        let bin1 = vec![1u8; 350 * 1024];
        let bin2 = vec![2u8; 400 * 1024];

        // Send the first binary message as a continuation (fragmented frames)
        // We'll split the message into 3 fragments and send as Continuation frames

        let bin1_chunks = bin1.chunks(120 * 1024).collect::<Vec<_>>();
        for (i, chunk) in bin1_chunks.iter().enumerate() {
            let is_last = i == bin1_chunks.len() - 1;
            let frame = if i == 0 {
                // First fragment: Binary, not final
                Message::Continuation(actix_ws::Item::FirstBinary(Bytes::from(chunk.to_vec())))
            } else {
                Message::Continuation(actix_ws::Item::Continue(Bytes::from(chunk.to_vec())))
            };
            framed.send(frame).await.expect("Failed to send bin1 fragment");
        }

        // Send the second binary message as a continuation (fragmented frames)
        let bin2_chunks = bin2.chunks(120 * 1024).collect::<Vec<_>>();
        let mut first = true;
        for (i, chunk) in bin2_chunks.iter().enumerate() {
            let is_last = i == bin2_chunks.len() - 1;
            let frame = if first {
                Message::Continuation(actix_ws::Item::FirstBinary(Bytes::from(chunk.to_vec())))
            } else if is_last {
                Message::Continuation(actix_ws::Item::Last(Bytes::from(chunk.to_vec())))
            } else {
                Message::Continuation(actix_ws::Item::Continue(Bytes::from(chunk.to_vec())))
            };
            framed.send(frame).await.expect("Failed to send bin2 fragment");
            first = false;
        }

        // Receive two "test" text messages in response
        let mut received = 0;
        while let Some(Ok(msg)) = framed.next().await {
            if let ws::Frame::Text(txt) = msg {
                assert_eq!(txt, "what a test");
                received += 1;
                if received == 2 {
                    break;
                }
            }
        }
        assert_eq!(received, 2, "Did not receive two 'test' messages");
    }

}
