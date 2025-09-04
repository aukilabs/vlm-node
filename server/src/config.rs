use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub model: String,
    pub ollama_host: String,
    pub image_batch_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Config {
            model: std::env::var("VLM_MODEL")?,
            ollama_host: std::env::var("OLLAMA_HOST")?,
            image_batch_size: std::env::var("IMAGE_BATCH_SIZE").unwrap_or("5".to_string()).parse::<usize>().unwrap()
        })
    }
}
