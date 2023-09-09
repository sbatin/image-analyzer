mod analyzer;
mod manager;
mod disjoint_set;

use analyzer::{Analyzer, Groups};
use manager::{TaskManager, TaskResponse};
use std::{
    path::PathBuf,
    sync::Arc,
};
use serde::{Serialize, Deserialize};
use eyre::Result;
use axum::{
    http::{Request, StatusCode},
    extract::{Query, State},
    routing::{get, get_service, post},
    response::{
        Json,
        sse::{Event, KeepAlive, Sse},
    },
    Router,
};
use tower::ServiceExt;
use tower_http::services;
use tokio::{
    task::JoinHandle,
    sync::{mpsc, oneshot, watch},
};
use futures::stream::{Stream, StreamExt};
use tokio_stream::wrappers::WatchStream;

#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
    dist: u32,
    path: PathBuf,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AnalyzeResponse {
    Pending { progress: usize },
    Completed { data: Groups },
    Failed { error: String },
}

type TaskResult = Result<Groups>;

enum AnalyzeCommand {
    Submit(AnalyzeRequest),
    Subscribe(PathBuf, oneshot::Sender<Option<watch::Receiver<usize>>>),
    Poll(PathBuf, oneshot::Sender<Option<TaskResponse<usize, TaskResult>>>),
}

async fn task_analyzer(mut rx: mpsc::Receiver<AnalyzeCommand>) {
    tracing::info!("manager task started");

    let engine = Arc::new(Analyzer::new());
    let mut manager: TaskManager<PathBuf, usize, TaskResult> = TaskManager::new();

    while let Some(command) = rx.recv().await {
        let engine2 = engine.clone();

        match command {
            AnalyzeCommand::Submit(req) => {
                tracing::info!("analyze task submit {:?}", req.path);
                manager.submit(req.path, move |path, tx| {
                    let data = engine2.analyze(&path, tx);
                    let groups = data.map(|data| {
                        /*let me = Arc::get_mut(&mut engine2);
                        if let Some(me) = me {
                            me.update_cache(&data);
                        } else {
                            tracing::warn!("cannot update analyzer cache");
                        }*/
                        analyzer::create_groups(&data, req.dist)
                    });
                    tracing::info!("analyze task completed {:?}", path);
                    groups
                }).await;
            }
            AnalyzeCommand::Subscribe(path, tx) => {
                let rx = manager.progress(&path);
                if let Err(_) = tx.send(rx) {
                    tracing::error!("unable to send response back to the client");
                }
            }
            AnalyzeCommand::Poll(path, tx) => {
                let resp = manager.poll(&path).await;
                if let Err(_) = tx.send(resp) {
                    tracing::error!("unable to send response back to the client");
                }    
            }
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
) -> Result<Json<bool>, StatusCode> {
    state
        .task_sender
        .send(AnalyzeCommand::Submit(req))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(true))
}

async fn poll(
    State(state): State<AppState>,
    Query(params): Query<PathParams>,
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand::Poll(params.path, tx))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let resp = resp.ok_or_else(|| StatusCode::NOT_FOUND)?;
    Ok(Json(match resp {
        TaskResponse::Pending(progress) => AnalyzeResponse::Pending { progress },
        TaskResponse::Completed(Ok(data)) => AnalyzeResponse::Completed { data },
        TaskResponse::Completed(Err(err)) => AnalyzeResponse::Failed { error: err.to_string() }
    }))
}

async fn subscribe(
    State(state): State<AppState>,
    Query(params): Query<PathParams>,
) -> Result<Sse<impl Stream<Item = Result<Event, serde_json::error::Error>>>, StatusCode> {
    tracing::info!("sse handler called {:?}", params.path);

    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand::Subscribe(params.path, tx))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let resp = resp.ok_or_else(|| StatusCode::NOT_FOUND)?;
    let stream = WatchStream::new(resp).map(|p| Event::default().json_data(p));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
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
        .route("/", get_service(services::ServeFile::new("client/dist/index.html")))
        .route("/image", get(|request: Request<_>| {
            // TODO: handle errors here
            let params: Query<PathParams> = Query::try_from_uri(request.uri()).unwrap();
            let service = services::ServeFile::new(&params.path);
            service.oneshot(request)
        }))
        .route("/list_folder", get(list_folder))
        .route("/analyze", post(analyze))
        .route("/poll", get(poll))
        .route("/subscribe", get(subscribe))
        .nest_service("/static", services::ServeDir::new("client/dist"))
        .nest_service("/assets", services::ServeDir::new("client/dist/assets"))
        .with_state(shared_state);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    tracing::info!("done");
    Ok(())
}