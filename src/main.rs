mod analyzer;
mod disjoint_set;

use analyzer::{AnalyzedData, Groups};
use std::{
    path::PathBuf,
    sync::Arc,
    collections::HashMap,
};
use serde::{Serialize, Deserialize};
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
    sync::{mpsc, oneshot, watch},
};

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
}

/// internal type used by task manager
enum TaskResponse<T> {
    Pending(usize),
    Completed(Result<T>),
}

impl<T> TaskResponse<T> {
    fn map<U, F: FnOnce(T) -> U>(self, op: F) -> TaskResponse<U> {
        match self {
            Self::Pending(progress) => TaskResponse::Pending(progress),
            Self::Completed(data) => TaskResponse::Completed(data.map(op)),
        }
    }
}

struct AnalyzeCommand {
    req: AnalyzeRequest,
    tx: oneshot::Sender<TaskResponse<Groups>>,
}

type TaskResult = Result<AnalyzedData>;

async fn task_analyzer(mut rx: mpsc::Receiver<AnalyzeCommand>) {
    tracing::info!("manager task started");

    let engine = Arc::new(analyzer::make_engine());
    let mut cache: HashMap<PathBuf, AnalyzedData> = HashMap::new();
    let mut tasks: HashMap<PathBuf, (JoinHandle<TaskResult>, watch::Receiver<usize>)> = HashMap::new();

    while let Some(command) = rx.recv().await {
        tracing::info!("analyze request received {:?}", command.req);

        let engine = engine.clone();

        let data: TaskResponse<&AnalyzedData> = match cache.get(&command.req.path) {
            Some(data) => TaskResponse::Completed(Ok(data)),
            None => {
                match tasks.remove(&command.req.path) {
                    Some((join_handle, mut rx)) => {
                        if let Ok(_) = rx.changed().await {
                            let progress = *rx.borrow();
                            tracing::info!(progress, "waiting for existing task {:?}", command.req.path);
                            // still in progress: put handles back to tasks
                            tasks.insert(command.req.path, (join_handle, rx));
                            TaskResponse::Pending(progress)
                        } else {
                            tracing::info!("existing task completed {:?}", command.req.path);
                            let result = join_handle.await.unwrap();
                            TaskResponse::Completed(result.map(|data| {
                                cache
                                    .entry(command.req.path)
                                    .or_insert(data) as &_
                            }))
                        }
                    }
                    None => {
                        tracing::info!("no task found, creating a new one {:?}", command.req.path);
                        let path = command.req.path.clone();
                        let (tx, rx) = watch::channel(0);
                        let join_handle = task::spawn_blocking(move || {
                            let result = analyzer::analyze_files(&engine, &path, tx);
                            tracing::info!("analyze task completed {:?}", path);
                            result
                        });
                        tasks.insert(command.req.path, (join_handle, rx));
                        TaskResponse::Pending(0)
                    }
                }
            }
        };

        let resp = data.map(|data| analyzer::create_groups(data, command.req.dist));
        if let Err(_) = command.tx.send(resp) {
            tracing::error!("unable to send response back to the client");
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
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand { req, tx })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = rx.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match result {
        TaskResponse::Pending(progress) => {
            Ok(Json(AnalyzeResponse::Pending { progress }))
        }
        TaskResponse::Completed(Ok(data)) => {
            Ok(Json(AnalyzeResponse::Completed { data }))
        }
        TaskResponse::Completed(Err(_)) => {
            tracing::error!("analyze task failed");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
        .nest_service("/static", services::ServeDir::new("client/dist"))
        .nest_service("/assets", services::ServeDir::new("client/dist/assets"))
        .with_state(shared_state);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    tracing::info!("done");
    Ok(())
}