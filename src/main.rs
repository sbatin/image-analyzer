mod analyzer;
mod manager;
mod cache;
mod disjoint_set;
mod remover;

use analyzer::{Analyzer, AnalyzeRequest, Groups, FileInfo};
use manager::{TaskManager, TaskResponse};
use remover::{Remover, RemovedFile};
use tracing::Span;
use std::{
    path::PathBuf,
    sync::Arc, time::{Instant, Duration}, convert::Infallible,
};
use serde::{Serialize, Deserialize};
use eyre::{Result, Report};
use axum::{
    http::{Request, StatusCode, Response},
    extract::{Query, State, Path},
    routing::{get, get_service, post},
    response::{
        Json, IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    Router,
};
use tower::ServiceExt;
use tower_http::{
    services,
    trace::TraceLayer,
};
use tokio::{
    task::JoinHandle,
    sync::{mpsc, oneshot, watch},
};
use futures::stream::{Stream, StreamExt};
use tokio_stream::wrappers::WatchStream;
use uuid::Uuid;

type TaskResult = Result<Groups>;

enum AnalyzeCommand {
    Submit(AnalyzeRequest, oneshot::Sender<Uuid>),
    Subscribe(Uuid, oneshot::Sender<Option<watch::Receiver<usize>>>),
    Poll(Uuid, oneshot::Sender<Option<TaskResponse<usize, TaskResult>>>),
}

async fn task_analyzer(mut rx: mpsc::Receiver<AnalyzeCommand>) {
    tracing::info!("manager task started");

    let engine = Arc::new(Analyzer::new());
    let mut manager: TaskManager<Uuid, usize, TaskResult> = TaskManager::new();

    while let Some(command) = rx.recv().await {
        match command {
            AnalyzeCommand::Submit(req, tx) => {
                tracing::info!("analyze task {:?} submitted", req);
                let engine = engine.clone();
                let task_id = Uuid::new_v4();
                manager.submit(task_id, move |tx| {
                    let started = Instant::now();
                    let result = engine.analyze(&req, tx);
                    let elapsed = started.elapsed();
                    tracing::info!("analyze task {:?} completed in {:?}", req, elapsed);
                    result
                });
                if tx.send(task_id).is_err() {
                    tracing::error!("unable to send response back to the client");
                }
            }
            AnalyzeCommand::Subscribe(task_id, tx) => {
                let rx = manager.progress(&task_id);
                if tx.send(rx).is_err() {
                    tracing::error!("unable to send response back to the client");
                }
            }
            AnalyzeCommand::Poll(task_id, tx) => {
                let resp = manager.poll(&task_id).await;
                if tx.send(resp).is_err() {
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

enum AppError {
    Internal(Report),
    Provided(StatusCode),
}

impl AppError {
    fn not_found() -> Self {
        Self::Provided(StatusCode::NOT_FOUND)
    }
}

impl<T> From<T> for AppError
where
    T: Into<Report>
{
    fn from(inner: T) -> Self {
        Self::Internal(inner.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Provided(code) => code,
        };
        status_code.into_response()
    }
}

type AppResult<T> = Result<T, AppError>;
type JsonResponse<T> = AppResult<Json<T>>;

struct AppState {
    task_sender: mpsc::Sender<AnalyzeCommand>,
    remover: Remover,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum AnalyzeResponse {
    Pending { progress: usize },
    Completed { data: Groups },
    Failed { error: String },
}

#[derive(Serialize, Deserialize)]
struct PathParams {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskParams {
    task_id: Uuid,
}

fn check_path(path: &std::path::Path) -> AppResult<()> {
    if !path.is_dir() {
        Err(AppError::not_found())
    } else {
        Ok(())
    }
}

async fn list_folder(Query(params): Query<PathParams>) -> JsonResponse<Vec<FileInfo>> {
    check_path(&params.path)?;

    let files = analyzer::list_dir(&params.path)?;
    Ok(Json(files))
}

async fn delete_file(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PathParams>,
) -> JsonResponse<String> {
    let base_name = state.remover.remove(&params.path)?;
    Ok(Json(base_name))
}

async fn restore_file(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> JsonResponse<PathBuf> {
    // TODO: check id

    let path = state.remover.restore(&id)?;
    Ok(Json(path))
}

async fn restore_all(
    State(state): State<Arc<AppState>>,
) -> AppResult<()> {
    state.remover.restore_all()?;
    Ok(())
}

async fn list_deleted(
    State(state): State<Arc<AppState>>,
) -> JsonResponse<Vec<RemovedFile>> {
    let files = state.remover.list_removed()?;
    Ok(Json(files))
}

async fn analyze(
    State(state): State<Arc<AppState>>,
    Query(req): Query<AnalyzeRequest>,
) -> JsonResponse<TaskParams> {
    check_path(&req.path)?;

    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand::Submit(req, tx))
        .await?;

    let task_id = rx.await?;

    Ok(Json(TaskParams { task_id }))
}

async fn poll(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TaskParams>,
) -> JsonResponse<AnalyzeResponse> {
    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand::Poll(params.task_id, tx))
        .await?;

    let resp = rx.await?;
    let resp = resp.ok_or_else(AppError::not_found)?;
    Ok(Json(match resp {
        TaskResponse::Pending(progress) => AnalyzeResponse::Pending { progress },
        TaskResponse::Completed(Ok(data)) => AnalyzeResponse::Completed { data },
        TaskResponse::Completed(Err(err)) => AnalyzeResponse::Failed { error: err.to_string() }
    }))
}

async fn subscribe(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TaskParams>,
) -> AppResult<Sse<impl Stream<Item = serde_json::error::Result<Event>>>> {
    tracing::info!("SSE handler called {:?}", params.task_id);

    let (tx, rx) = oneshot::channel();

    state
        .task_sender
        .send(AnalyzeCommand::Subscribe(params.task_id, tx))
        .await?;

    let resp = rx.await?;
    let resp = resp.ok_or_else(AppError::not_found)?;
    let stream = WatchStream::new(resp).map(|p| Event::default().json_data(p));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

type FileResponse = Response<tower_http::services::fs::ServeFileSystemResponseBody>;

async fn serve_image<T>(
    Query(params): Query<PathParams>,
    request: Request<T>,
) -> Result<FileResponse, Infallible>
where
    T: Send + 'static
{
    let service = services::ServeFile::new(&params.path);
    service.oneshot(request).await
}

async fn serve_deleted<T>(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    request: Request<T>,
) -> AppResult<FileResponse>
where
    T: Send + 'static
{
    let path = state.remover.resolve(&id)?;
    let service = services::ServeFile::new(&path);
    let response = service.oneshot(request).await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting...");

    let (_, task_sender) = spawn_analyzer();
    let remover = Remover::new("removed");
    let shared_state = Arc::new(AppState { task_sender, remover });

    let http_logger = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<_>| {
            let path = req.uri().path();
            let method = req.method().as_str();
            let status = tracing::field::Empty;
            tracing::info_span!("http", method, path, status)
        })
        .on_response(|resp: &Response<_>, elapsed: Duration, span: &Span| {
            let status = resp.status().as_u16();
            span.record("status", status);
            let level = if status >= 500 {
                log::Level::Error
            } else if status >= 400 {
                log::Level::Warn
            } else {
                log::Level::Info
            };
            // tracing doesn't accept dynamic log levels
            log::log!(level, "completed in {:?}", elapsed);
        });

    let app = Router::new()
        .route("/", get_service(services::ServeFile::new("client/dist/index.html")))
        .route("/image", get(serve_image))
        .route("/list_folder", get(list_folder))
        .route("/delete_file", post(delete_file))
        .route("/deleted", get(list_deleted))
        .route("/deleted/:id", get(serve_deleted))
        .route("/deleted/:id/restore", post(restore_file))
        .route("/deleted/restore_all", post(restore_all))
        .route("/analyze", post(analyze))
        .route("/poll", get(poll))
        .route("/subscribe", get(subscribe))
        .nest_service("/static", services::ServeDir::new("client/dist"))
        .nest_service("/assets", services::ServeDir::new("client/dist/assets"))
        .with_state(shared_state)
        .layer(http_logger);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    tracing::info!("done");
    Ok(())
}
