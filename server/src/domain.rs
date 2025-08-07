use std::path::Path;

use posemesh_domain_http::{domain_data::DownloadQuery, DomainClient};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

pub async fn download_for_job(
    domain_client: &DomainClient,
    job_id: &str,
    domain_id: &str,
    data_dir: &str,
    query: &DownloadQuery,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut count = 0;
    let mut rx = domain_client.download_domain_data(&domain_id, query).await?;

    while let Some(Ok(data)) = rx.next().await {
        let dir_path = format!("{}/input/{}", data_dir, job_id);
        if let Err(e) = fs::create_dir_all(&dir_path).await {
            tracing::error!("Failed to create directory {}: {:?}", dir_path, e);
            rx.close();
            return Err(e.into());
        }

        let file_name = format!("{}_{}.{}", data.id, data.name, data.data_type);
        let file_path = Path::new(&dir_path).join(file_name);

        // Assume data.data is a Vec<u8> or something that can be written as bytes
        match fs::File::create(&file_path).await {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&data.data).await {
                    tracing::error!("Failed to write data to file {:?}: {:?}", file_path, e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to create file {:?}: {:?}", file_path, e);
                return Err(e.into());
            }
        }
        count += 1;
    }

    Ok(count)
}
