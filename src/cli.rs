mod data;

use axum::http::{HeaderMap, HeaderValue};
use reqwest::{
    header::{CONTENT_DISPOSITION, CONTENT_TYPE, SERVER},
    Client,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::path::Path;

use crate::data::*;

pub const SERVER_ADDRESS: &'static str = "http://192.168.50.116:80";
pub const UPLOAD_FILE_NAME_TEMP: &'static str = "C:/s.zip";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("Hello from Client!");

    let id = register().await?;
    println!("SUCCESS! Registered with ID: {}\n\n", id);

    let upload_message = upload(UPLOAD_FILE_NAME_TEMP, &id).await?;
    println!("Tried to upload! Server says: `{}`.\n\n", upload_message);

    let files = list(&id).await?;
    println!("{}\n\n", files);

    Ok(())
}

async fn register() -> Result<Uuid, reqwest::Error> {
    let smth = Client::new()
        .get(format!("{}/register", SERVER_ADDRESS))
        .send()
        .await?
        .json::<RegisteredResponse>()
        .await?;

    Ok(smth.id)
}

async fn list(id: &Uuid) -> Result<ListFilesResponse, reqwest::Error> {
    Ok(Client::new()
        .get(format!("{}/list/{}", SERVER_ADDRESS, id.to_string()))
        .send()
        .await?
        .json::<ListFilesResponse>()
        .await?)
}

async fn upload(filename: &str, id: &Uuid) -> Result<String, reqwest::Error> {
    let url_filename = Path::new(filename)
        .file_name()
        .expect("ERROR on UPLOAD: corrupted file name!")
        .to_str()
        .unwrap()
        .to_owned();
    let the_file = tokio::fs::read(filename).await.unwrap();
    let mut header_map = HeaderMap::new();
    header_map.insert(
        CONTENT_TYPE,
        HeaderValue::from_str("multipart/form-data").unwrap(),
    );
    header_map.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename={}", url_filename)).unwrap(),
    );
    let file_part = reqwest::multipart::Part::bytes(the_file)
        .file_name(url_filename.clone())
        .headers(header_map);
    let form = reqwest::multipart::Form::new()
        .text("filename", url_filename.clone())
        .part("multipart", file_part);
    let message = Client::new()
        .post(format!(
            "{}/upload/{}/{}",
            SERVER_ADDRESS,
            id.to_string(),
            &url_filename
        ))
        .multipart(form)
        .send()
        .await?
        .text()
        .await?;

    Ok(message)
}
