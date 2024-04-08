use axum::{
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use std::{
    collections::HashMap,
    path::Path as P,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// TODO: Develop an actual client app so that you don't have to send all requests via curl.

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spfiler=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Compose the routes
    // TODO: Download functionality
    // TODO: Deleting files from ID
    // TODO: Secure downloads/uploads via HTTPS protocol
    let app = Router::new()
        .route("/kys", get(exit_app))
        .route("/register", get(register_id))
        .route("/list/:registered_id", get(list_files))
        .route("/upload/:id/:filename", post(upload_file))
        // Add middleware to all routes
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(1 * 1024 * 1024 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(FileCoordinator::new_async());

    let listener = tokio::net::TcpListener::bind("192.168.50.116:80")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
// The query for exiting the app
pub struct ExitResponse {
    response: String,
}

fn do_exit() {
    std::thread::sleep(Duration::from_millis(300));
    std::process::exit(0);
}

async fn exit_app() -> impl IntoResponse {
    tokio::spawn(async move {
        do_exit();
    });
    (
        StatusCode::OK,
        Json(ExitResponse {
            response: "Yeah, it wasn't all that fun anyway...".to_owned(),
        }),
    )
}

#[derive(Serialize, Debug)]
pub struct RegisteredResponse {
    pub id: String,
    pub message: String,
}

async fn register_id(State(files): State<Files>) -> impl IntoResponse {
    let registered_id = Uuid::new_v4();
    files.write().unwrap().list.insert(registered_id, vec![]);

    (StatusCode::CREATED, Json(RegisteredResponse {
        id: registered_id.to_string(),
        message: "Your new file sharing ID has been registered! Do NOT lose this ID, it is your key to sharing files via this app!".to_owned(),
    }))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ListFilesResponse {
    message: String,
    files: Option<Vec<String>>,
}

async fn list_files(
    Path(registered_id): Path<Uuid>,
    State(files): State<Files>,
) -> impl IntoResponse {
    let maybe_files = files
        .read()
        .unwrap()
        .list
        .get(&registered_id)
        .map(|o| o.clone());

    match maybe_files {
        None => (
            StatusCode::BAD_REQUEST,
            Json(ListFilesResponse {
                message: "Sorry, no such ID has been registered yet!".to_owned(),
                files: None,
            }),
        ),
        Some(file_vec) => (
            StatusCode::BAD_REQUEST,
            Json(ListFilesResponse {
                message: format!("Found files for id {}!", registered_id.to_string()),
                files: Some(file_vec),
            }),
        ),
    }
}

// TODO: Sanitize uploaded file names, make sure names are sane
// TODO: Rethink the URL structure in order to not expose the file names in the URL for security
// TODO: Maybe handle cases if the connection is dropped and the file ends up being only partially uploaded?
// TODO: Could also explore streaming files and not uploading them in a single operation (for larger files).
async fn upload_file(
    Path((id, filename)): Path<(Uuid, String)>,
    State(files): State<Files>,
    mut mp: Multipart,
) -> (StatusCode, String) {
    if files.read().unwrap().list.get(&id).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            "ERROR: No such ID has been registered, can't upload any files!".to_owned(),
        );
    }

    while let Some(field) = mp.next_field().await.unwrap() {
        let prefix = files.read().unwrap().storage_prefix.clone();
        let name = field.name().unwrap().to_string();
        let filename_async = field.file_name().unwrap().to_string();

        if name == "filename".to_string() && filename_async == filename {
            info!("UPLOADING: prefix: `{}`, name: `{}`, filename in field: `{}`, filename in request: `{}`.", &prefix, &name, &filename_async, &filename);
            let data = field.bytes().await.unwrap();

            let dirpath = P::new(&prefix).join(&id.to_string());
            if !tokio::fs::try_exists(&dirpath).await.unwrap() {
                tokio::fs::create_dir_all(&dirpath).await.unwrap();
            }

            match tokio::fs::write(dirpath.join(&filename), data).await {
                Ok(()) => {
                    let mut coord = files.write().unwrap();
                    let id_files = coord.list.get_mut(&id).unwrap();
                    id_files.push(filename);
                    return (StatusCode::OK, "File uploaded!".to_owned());
                }
                Err(e) => return (
                    StatusCode::BAD_GATEWAY,
                    format!("Couldn't upload chunk for this reason: `{}`, path being written to: `{:?}`", e, dirpath.join(&filename).to_str()),
                ),
            };
        }
    }

    (
        StatusCode::BAD_REQUEST,
        "Received request to upload, but couldn't start receiving data?..".to_owned(),
    )
}

// TODO: Save coordinator's state to a JSON so that it remembers its state between sessions
pub struct FileCoordinator {
    pub storage_prefix: String,
    pub list: HashMap<Uuid, Vec<String>>,
}

impl FileCoordinator {
    pub fn new_async() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            storage_prefix: "files".to_owned(),
            list: HashMap::new(),
        }))
    }
}

type Files = Arc<RwLock<FileCoordinator>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}
