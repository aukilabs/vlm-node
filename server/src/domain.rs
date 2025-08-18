use std::path::Path;

use futures::channel::mpsc::{self, Sender};
use posemesh_domain_http::domain_data::{CreateDomainData, DomainData, UploadDomainData};
use posemesh_domain_http::{domain_data::DownloadQuery, DomainClient};
use tokio::fs::read_dir;
use tokio::{fs, spawn};
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

pub async fn download_for_job(
    domain_client: &DomainClient,
    job_id: &str,
    domain_id: &str,
    data_dir: &str,
    query: &DownloadQuery,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let mut count = 0;
    let mut rx = domain_client.download_domain_data(&domain_id, query).await?;

    while let Some(Ok(data)) = rx.next().await {
        let dir_path = format!("{}/input/{}", data_dir, job_id);
        if let Err(e) = fs::create_dir_all(&dir_path).await {
            tracing::error!("Failed to create directory {}: {:?}", dir_path, e);
            rx.close();
            return Err(e.into());
        }

        let file_name = format!("{}_{}.{}", data.name, data.id, data.data_type);
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

pub async fn upload_for_job(
    domain_client: &DomainClient,
    domain_id: &str,
    data_dir: &str,
) -> Result<Vec<DomainData>, Box<dyn std::error::Error + Send + Sync>> {
    use futures::SinkExt;
    let (mut tx, rx) = mpsc::channel::<UploadDomainData>(100);

    let data_dir = data_dir.to_string();
    spawn(async move {
        let mut dir = read_dir(data_dir).await.expect("Failed to read data directory");
        while let Ok(Some(file)) = dir.next_entry().await {
            let file_path = file.path();
            let file_name = file_path.file_name().expect("Failed to get file name").to_str().expect("Failed to convert file name to string");
            let file_ext = file_path.extension().expect("Failed to get file extension").to_str().expect("Failed to convert file extension to string");
            tx.send(UploadDomainData {
                create: Some(CreateDomainData {
                    name: file_name.to_string(),
                    data_type: file_ext.to_string(),
                }),
                update: None,
                data: fs::read(file_path).await.expect("Failed to read file")
            }).await.expect("Failed to send file to channel");
        }
        tx.close();
    });
    let res = domain_client.upload_domain_data(domain_id, rx).await?;
    Ok(res)
}
