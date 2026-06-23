//! `leaf serve` — a read-only local web UI over the `.leaf` workspace.
//!
//! The server reuses the same Rust parsers as the CLI (`inventory::load`,
//! `review::build_json`) so `.leaf` stays the single source of truth; the web
//! UI is a lens, never a second writer. There are deliberately no mutating
//! routes — fall / checkpoint / gate advance stay in the CLI.

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use std::net::SocketAddr;
use std::path::{Path as StdPath, PathBuf};
#[cfg(not(feature = "embed-web"))]
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    repo_root: PathBuf,
}

/// Entry point for the `Serve` command. Builds a single-threaded-friendly
/// multi-thread runtime only here, so the rest of the CLI stays synchronous.
pub(crate) fn run(port: u16) -> Result<()> {
    let repo = crate::git::repo_paths(std::env::current_dir()?)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    runtime.block_on(serve(repo.root, port))
}

async fn serve(repo_root: PathBuf, port: u16) -> Result<()> {
    let web_dist = repo_root.join("web").join("dist");
    let state = AppState { repo_root };

    let app = Router::new()
        .route("/api/review/{slug}", get(review_handler))
        .route("/api/list", get(list_handler))
        .route("/api/graph", get(graph_handler));
    let app = attach_static(app, &web_dist).with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr} (is the port already in use?)"))?;
    println!("leaf serve (read-only) on http://{addr}");
    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}

async fn review_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let repo_root = state.repo_root.clone();
    match blocking(move || build_review(&repo_root, &slug)).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(err) => json_error(StatusCode::NOT_FOUND, &err.to_string()),
    }
}

fn build_review(repo_root: &StdPath, slug: &str) -> Result<crate::review::ReviewJson> {
    let slug = crate::slug::validate(slug)?;
    let inventory = crate::inventory::load(repo_root)?;
    let source = crate::review::source_for_slug(&inventory, &slug)?;
    crate::review::build_json(&source)
}

/// Workspace list as JSON. Reuses the exact `leaf list --json` projection, so
/// CLI and web list payloads share one typed shape.
async fn list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo_root = state.repo_root.clone();
    match blocking(move || {
        let inventory = crate::inventory::load(&repo_root)?;
        crate::list_output::JsonInventory::from_inventory(&inventory)
    })
    .await
    {
        Ok(output) => (StatusCode::OK, Json(output)).into_response(),
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
    }
}

async fn blocking<T, F>(work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .context("join blocking task")?
}

/// Pressed knowledge graph as JSON (`leaf graph --json` payload).
async fn graph_handler(State(state): State<AppState>) -> impl IntoResponse {
    let repo_root = state.repo_root.clone();
    match blocking(move || crate::graph::load(&repo_root)).await {
        Ok(graph) => (StatusCode::OK, Json(graph)).into_response(),
        Err(err) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &err.to_string()),
    }
}

fn json_error(code: StatusCode, message: &str) -> axum::response::Response {
    (code, Json(serde_json::json!({ "error": message }))).into_response()
}

/// Attach the SPA static layer. With `embed-web`, assets are baked into the
/// binary (standalone `leaf serve`); otherwise they are served from `web/dist`
/// on disk, with a dev notice when it has not been built yet.
#[cfg(not(feature = "embed-web"))]
fn attach_static(app: Router<AppState>, web_dist: &StdPath) -> Router<AppState> {
    if web_dist.is_dir() {
        app.fallback_service(ServeDir::new(web_dist))
    } else {
        app.fallback(dev_notice)
    }
}

#[cfg(feature = "embed-web")]
fn attach_static(app: Router<AppState>, _web_dist: &StdPath) -> Router<AppState> {
    app.fallback(embedded_handler)
}

#[cfg(feature = "embed-web")]
#[derive(rust_embed::RustEmbed)]
#[folder = "web/dist"]
struct WebAssets;

#[cfg(feature = "embed-web")]
async fn embedded_handler(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let asset = WebAssets::get(path).or_else(|| WebAssets::get("index.html"));
    match asset {
        Some(content) => {
            let mime = mime_guess_for(path);
            ([("content-type", mime)], content.data.into_owned()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

#[cfg(feature = "embed-web")]
fn mime_guess_for(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}

#[cfg(not(feature = "embed-web"))]
async fn dev_notice() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/html; charset=utf-8")],
        "<h1>leaf serve</h1><p><code>web/dist</code> not built. Run \
         <code>cd web &amp;&amp; npm install &amp;&amp; npm run build</code>, or use the \
         Vite dev server which proxies <code>/api</code> here.</p>",
    )
}
