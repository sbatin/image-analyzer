mod analyzer;

use std::{
    path::PathBuf,
    sync::Arc,
    collections::HashMap,
};
use image_hasher::{HasherConfig, HashAlg};
use serde::Deserialize;
use eyre::Result;
use axum::{
    http::{Request, StatusCode},
    extract::{Query, State},
    routing::{get, get_service, post},
    response::Json,
    Router,
};
use tower::ServiceExt;
use tower_http::services;
use tokio::{
    task::{self, JoinHandle},
    sync::{mpsc, oneshot},
};

#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
    dist: u32,
    path: PathBuf,
}

struct AnalyzeCommand {
    req: AnalyzeRequest,
    tx: oneshot::Sender<Vec<Vec<PathBuf>>>,
}

async fn task_analyzer(mut rx: mpsc::Receiver<AnalyzeCommand>) {
    tracing::info!("manager task started");

    let hasher = Arc::new(HasherConfig::new()
        .hash_size(16, 16)
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher());

    let mut cache = HashMap::new();

    while let Some(command) = rx.recv().await {
        tracing::info!("analyze request received {:?}", command.req);

        let hasher = hasher.clone();

        let hashes = match cache.get(&command.req.path) {
            Some(data) => Ok(data),
            None => {
                let path = command.req.path.clone();
                let task_result = task::spawn_blocking(move || {
                    let result = analyzer::analyze_files(&hasher, &path);
                    tracing::info!("analyze task completed {:?}", path);
                    result
                }).await.unwrap();

                task_result.map(|hashes| {
                    cache
                        .entry(command.req.path)
                        .or_insert(hashes) as &analyzer::Hashes
                })
            }
        };

        if let Ok(hashes) = hashes {
            let result = analyzer::create_groups(hashes, command.req.dist);
            if let Err(_) = command.tx.send(result) {
                tracing::error!("unable to send response back to the client");
            }
        } else {
            tracing::error!("analyze task failed");
        }
    }

    tracing::info!("manager task exiting");
}

fn spawn_analyzer() -> (JoinHandle<()>, mpsc::Sender<AnalyzeCommand>) {
    let (tx, rx) = mpsc::channel(32);
    let join_handle = tokio::spawn(task_analyzer(rx));
    (join_handle, tx)
}

#[derive(Deserialize)]
struct PathParams {
    path: PathBuf,
}

async fn list_folder(Query(params): Query<PathParams>) -> Json<Vec<PathBuf>> {
    let files = analyzer::list_dir(&params.path);
    Json(files)
}

async fn analyze(
    State(state): State<AppState>,
    Query(req): Query<AnalyzeRequest>,
) -> Result<Json<Vec<Vec<PathBuf>>>, StatusCode> {
    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand { req, tx })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}

#[derive(Clone)]
struct AppState {
    task_sender: mpsc::Sender<AnalyzeCommand>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting...");

    let (_, task_sender) = spawn_analyzer();
    let shared_state = AppState { task_sender };

    let app = Router::new()
        .route("/", get_service(services::ServeFile::new("static/index.html")))
        .route("/image", get(|request: Request<_>| {
            // TODO: handle errors here
            let params: Query<PathParams> = Query::try_from_uri(request.uri()).unwrap();
            let service = services::ServeFile::new(&params.path);
            service.oneshot(request)
        }))
        .route("/images", get_service(services::ServeFile::new("static/images.html")))
        .route("/list_folder", get(list_folder))
        .route("/analyze", post(analyze))
        .nest_service("/static", services::ServeDir::new("static"))
        .with_state(shared_state);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    tracing::info!("done");
    Ok(())
}