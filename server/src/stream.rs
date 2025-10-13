use actix_web::{rt, web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::{select, FutureExt};
use futures_util::StreamExt as _;
use serde_json::Value;
use tokio::time::{self, Duration, Instant};

use crate::{config, ollama_client::{send_to_ollama}};

async fn proxy_ollama_response(
    session: &mut actix_ws::Session,
    images_batch: Vec<Vec<u8>>,
    prompt: String,
    model: String,
    ollama_host: String,
    num_predict: i32,
) {
    let mut session_clone = session.clone();
    rt::spawn(async move {
        match send_to_ollama(images_batch, prompt, model, ollama_host, Some(num_predict)).await {
            Ok(mut rx) => {
                while let Some(res) = rx.next().await {
                    match res {
                        Ok(bytes) => {
                            if let Err(e) = session_clone.binary(bytes).await {
                                tracing::error!("Error: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error: {:?}", e);
            }
        }
    });
}

async fn handle_binary(
    images: &mut Vec<Vec<u8>>,
    bin: bytes::Bytes,
    session: &mut actix_ws::Session,
    last_prompt: &Option<(String, i32)>,
    model: String,
    ollama_host: String,
    image_batch_size: usize,
) {
    images.push(bin.to_vec());

    if images.len() >= image_batch_size {
        if let Some((prompt, num_predict)) = last_prompt.clone() {
            let images_batch = std::mem::take(images);
            proxy_ollama_response(session, images_batch, prompt, model, ollama_host, num_predict).await;
        } else {
            let _ = session.text("No prompt received for image batch".to_string()).await;
        }
    }
}

async fn handle_text(
    session: &mut actix_ws::Session,
    images: &mut Vec<Vec<u8>>,
    text: String,
    last_prompt: &mut Option<(String, i32)>,
    model: String,
    ollama_host: String,
) {
    let (prompt, num_predict) = if let Ok(value) = serde_json::from_str::<Value>(&text) {
        let prompt = value["prompt"].as_str().unwrap_or(&text).to_string();
        let num_predict = value["num_predict"].as_i64().map(|n| n as i32).unwrap_or(16);
        (prompt, num_predict)
    } else {
        (text.clone(), 16)
    };

    *last_prompt = Some((prompt.clone(), num_predict));

    // If prompt updated, send the images to Ollama
    if !images.is_empty() {
        let images_batch = std::mem::take(images);
        proxy_ollama_response(session, images_batch, prompt, model, ollama_host, num_predict).await;
    }
}

async fn handle_ping(session: &mut actix_ws::Session, msg: bytes::Bytes) {
    let _ = session.pong(&msg).await;
}

pub async fn ws_index(req: HttpRequest, stream: web::Payload, vlm_config: web::Data<config::Config>) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let ollama_host = vlm_config.ollama_host.clone();
    let model = vlm_config.model.clone();
    let image_batch_size = vlm_config.image_batch_size;

    let mut stream = stream.max_frame_size(1024*1024);

    rt::spawn(async move {
        let mut images: Vec<Vec<u8>> = Vec::new();
        let mut last_prompt: Option<(String, i32)> = None;

        let mut inference_interval = time::interval_at(Instant::now() + Duration::from_secs(30), Duration::from_secs(10));
        inference_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            select! {
                msg = stream.next().fuse() => {
                    match msg {
                        Some(Ok(Message::Binary(bin))) => {
                            tracing::info!("Received binary message: {:?}", bin.len());
                            inference_interval.reset();
                            handle_binary(&mut images, bin, &mut session, &last_prompt, model.clone(), ollama_host.clone(), image_batch_size).await;
                        }
                        Some(Ok(Message::Text(text))) => {
                            tracing::info!("Received text message: {:?}", text);
                            inference_interval.reset();
                            handle_text(&mut session, &mut images, text.to_string(), &mut last_prompt, model.clone(), ollama_host.clone()).await;
                        }
                        Some(Ok(Message::Close(_))) => {
                            tracing::info!("Received close message");
                            break;
                        }
                        Some(Ok(Message::Ping(msg))) => {
                            inference_interval.reset();
                            handle_ping(&mut session, msg).await;
                            tracing::info!("Received ping message");
                        }
                        Some(Ok(Message::Pong(_))) => {
                            inference_interval.reset();
                            tracing::info!("Received pong message");
                        }
                        Some(Err(e)) => {
                            tracing::error!("Received message error: {:?}", e);
                        }
                        Some(Ok(res)) => {
                            tracing::info!("Received unknown message: {:?}", res);
                        }
                        None => {
                            tracing::info!("Received none message");
                            break;
                        }
                    }
                }
                _ = inference_interval.tick().fuse() => {
                    if let Err(e) = session.ping(b"ping").await {
                        tracing::error!("Error sending ping: {:?}", e);
                        break;
                    }
                    if let Some((prompt, num_predict)) = last_prompt.clone() {
                        if !images.is_empty() {
                            let images_batch = std::mem::take(&mut images);
                            proxy_ollama_response(&mut session, images_batch, prompt, model.clone(), ollama_host.clone(), num_predict).await;
                        }
                    }
                }
            }
        }

        tracing::info!("Stream closed");
    });

    Ok(res)
}
