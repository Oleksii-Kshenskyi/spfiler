mod data;

use reqwest::{header::SERVER, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data::*;

pub const SERVER_ADDRESS: &'static str = "http://192.168.50.116:80";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    println!("Hello from Client!");

    let id = register().await?;
    println!("SUCCESS! Registered with ID: {}\n\n", id);

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
