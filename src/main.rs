mod analyzer;

use std::path::PathBuf;
use image_hasher::{Hasher, HasherConfig, HashAlg};
use axum::{
    http::Request,
    extract::Query,
    routing::{get, get_service, post},
    response::Json,
    Router,
};
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
};
use serde::Deserialize;
use eyre::Result;
use tokio::task;

#[derive(Deserialize)]
struct PathParams {
    path: PathBuf,
}

async fn list_folder(Query(params): Query<PathParams>) -> Json<Vec<PathBuf>> {
    let files = analyzer::list_dir(&params.path);
    Json(files)
}

#[derive(Deserialize)]
struct AnalyzeParams {
    dist: u32,
    path: PathBuf,
}

async fn analyze(Query(params): Query<AnalyzeParams>) -> Json<Vec<Vec<PathBuf>>> {
    let hasher = HasherConfig::new()
        .hash_size(16, 16)
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher();

    let join_handle = task::spawn_blocking(move || {
        analyzer::analyze_files(&hasher, &params.path)
    });

    let hashes = join_handle.await.unwrap().unwrap();

    let result = analyzer::create_groups(&hashes, params.dist);
    Json(result)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    tracing::info!("starting...");

    let app = Router::new()
        .route("/", get_service(ServeFile::new("static/index.html")))
        .route("/image", get(|request: Request<_>| {
            // TODO: handle errors here
            let params: Query<PathParams> = Query::try_from_uri(request.uri()).unwrap();
            let service = ServeFile::new(&params.path);
            service.oneshot(request)
        }))
        .route("/images", get_service(ServeFile::new("static/images.html")))
        .route("/list_folder", get(list_folder))
        .route("/analyze", post(analyze))
        .nest_service("/static", ServeDir::new("static"));

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    /*let args: Vec<_> = env::args().collect();
    println!("starting {:?}", args);
    let dir1 = Path::new(&args[1]);
    let dir2 = Path::new(&args[2]);
    let remove = args.get(3).map(|s| s as &str) == Some("remove");
    let mut files = HashMap::new();
    let _ = check_dirs(&hasher, dir2, &mut files, dir2, remove);*/
    tracing::info!("done");
    Ok(())
}
