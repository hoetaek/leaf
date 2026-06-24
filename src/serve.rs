//! `leaf serve` — a read-only local web UI over the `.leaf` workspace.
//!
//! The server reuses the same Rust parsers as the CLI (`inventory::load`,
//! `review::build_json`) so `.leaf` stays the single source of truth; the web
//! UI is a lens, never a second writer. There are deliberately no mutating
//! routes — fall / checkpoint / gate advance stay in the CLI.

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::path::{Path as StdPath, PathBuf};
#[cfg(not(feature = "embed-web"))]
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    repo_root: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PortFallback {
    Auto,
    Strict,
}

#[derive(Debug)]
struct BoundListener {
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
    preferred_port_was_busy: bool,
}

/// Entry point for the `Serve` command. Builds a single-threaded-friendly
/// multi-thread runtime only here, so the rest of the CLI stays synchronous.
pub(crate) fn run(port: u16, fallback: PortFallback) -> Result<()> {
    let repo = crate::git::repo_paths(std::env::current_dir()?)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    runtime.block_on(serve(repo.root, port, fallback))
}

async fn serve(repo_root: PathBuf, port: u16, fallback: PortFallback) -> Result<()> {
    let web_dist = web_dist_dir(&repo_root);
    let state = AppState { repo_root };

    let app = Router::new()
        .route("/api/review/{slug}", get(review_handler))
        .route("/api/preview/{slug}", get(preview_handler))
        .route("/api/list", get(list_handler))
        .route("/api/graph", get(graph_handler));
    let app = attach_static(app, &web_dist).with_state(state);

    let bound = bind_preferred_listener(port, fallback).await?;
    if bound.preferred_port_was_busy {
        println!("port {port} is busy; using {}", bound.addr.port());
    }
    let addr = bound.addr;
    let listener = bound.listener;
    println!("leaf serve (read-only) on http://{addr}");
    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}

async fn bind_preferred_listener(
    preferred_port: u16,
    fallback: PortFallback,
) -> Result<BoundListener> {
    let preferred_addr = localhost(preferred_port);
    match tokio::net::TcpListener::bind(preferred_addr).await {
        Ok(listener) => {
            return Ok(BoundListener {
                listener,
                addr: preferred_addr,
                preferred_port_was_busy: false,
            });
        }
        Err(err) if fallback == PortFallback::Auto && err.kind() == ErrorKind::AddrInUse => {}
        Err(err) => {
            return Err(err)
                .with_context(|| format!("bind {preferred_addr} (is the port already in use?)"));
        }
    }

    for port in preferred_port.saturating_add(1)..=u16::MAX {
        let addr = localhost(port);
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                return Ok(BoundListener {
                    listener,
                    addr,
                    preferred_port_was_busy: true,
                });
            }
            Err(err) if err.kind() == ErrorKind::AddrInUse => continue,
            Err(err) => return Err(err).with_context(|| format!("bind {addr}")),
        }
    }

    bail!("bind {preferred_addr} (is the port already in use?)")
}

fn localhost(port: u16) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], port))
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

async fn preview_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let repo_root = state.repo_root.clone();
    match blocking(move || build_preview(&repo_root, &slug)).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(err) => json_error(StatusCode::NOT_FOUND, &err.to_string()),
    }
}

fn build_preview(repo_root: &StdPath, slug: &str) -> Result<crate::preview::PreviewJson> {
    let slug = crate::slug::validate(slug)?;
    let inventory = crate::inventory::load(repo_root)?;
    let item = inventory
        .stages
        .iter()
        .flat_map(|stage| stage.items.iter())
        .find(|item| {
            item.kind == crate::inventory::ItemKind::LeafWork && item.slug.as_str() == slug
        })
        .with_context(|| format!("leaf work not found: {slug}"))?;
    crate::preview::build_json_from_source(&item.slug, &item.preview)
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

#[cfg(not(feature = "embed-web"))]
fn web_dist_dir(_served_repo_root: &StdPath) -> PathBuf {
    leaf_web_dir().join("dist")
}

#[cfg(feature = "embed-web")]
fn web_dist_dir(_served_repo_root: &StdPath) -> PathBuf {
    PathBuf::new()
}

/// Non-embedded builds are for leaf contributors: serve the web UI from the
/// leaf checkout that built this binary, not from the project being viewed.
/// Installed release binaries should use `embed-web` instead.
#[cfg(not(feature = "embed-web"))]
fn leaf_web_dir() -> PathBuf {
    StdPath::new(env!("CARGO_MANIFEST_DIR")).join("web")
}

/// Attach the SPA static layer. With `embed-web`, assets are baked into the
/// binary (standalone `leaf serve`); otherwise they are served from the leaf
/// source checkout's `web/dist`, with a dev notice when it has not been built.
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
        dev_notice_html(),
    )
}

#[cfg(not(feature = "embed-web"))]
fn dev_notice_html() -> String {
    let leaf_web_dir = html_escape(&leaf_web_dir().display().to_string());
    format!(
        "<h1>leaf serve</h1>\
         <p>This developer leaf binary was built without embedded web UI, and the leaf source checkout has no <code>web/dist</code>.</p>\
         <p>Build the leaf web UI from the leaf source checkout, not the project you are serving:</p>\
         <pre><code>cd {leaf_web_dir} &amp;&amp; npm ci &amp;&amp; npm run build</code></pre>\
         <p>For UI development, run <code>npm run dev</code> from that same <code>web</code> directory and open the Vite URL.</p>\
         <p>Installed release binaries should be built with embedded web UI, so they do not need this source checkout.</p>"
    )
}

#[cfg(not(feature = "embed-web"))]
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::net::TcpListener as StdTcpListener;

    #[tokio::test]
    async fn bind_preferred_listener_falls_back_when_preferred_port_is_busy() {
        let occupied = StdTcpListener::bind(("127.0.0.1", 0)).expect("bind occupied port");
        let occupied_port = occupied.local_addr().expect("occupied local addr").port();

        let bound = bind_preferred_listener(occupied_port, PortFallback::Auto)
            .await
            .expect("fallback bind");

        assert_ne!(bound.addr.port(), occupied_port);
        assert!(bound.preferred_port_was_busy);
    }

    #[tokio::test]
    async fn bind_preferred_listener_reports_busy_port_when_strict() {
        let occupied = StdTcpListener::bind(("127.0.0.1", 0)).expect("bind occupied port");
        let occupied_port = occupied.local_addr().expect("occupied local addr").port();

        let result = bind_preferred_listener(occupied_port, PortFallback::Strict).await;

        assert!(result.is_err());
        assert!(
            result
                .expect_err("strict bind should fail")
                .to_string()
                .contains("is the port already in use?")
        );
    }

    #[cfg(not(feature = "embed-web"))]
    #[test]
    fn web_dist_dir_uses_leaf_checkout_not_served_repo() {
        let served_repo = tempfile::tempdir().expect("served repo tempdir");

        let web_dist = web_dist_dir(served_repo.path());

        assert_eq!(
            web_dist,
            StdPath::new(env!("CARGO_MANIFEST_DIR"))
                .join("web")
                .join("dist")
        );
    }

    #[cfg(not(feature = "embed-web"))]
    #[test]
    fn dev_notice_points_to_leaf_checkout_not_served_project() {
        let html = dev_notice_html();

        assert!(html.contains("leaf source checkout"));
        assert!(html.contains("not the project you are serving"));
        assert!(html.contains("release"));
        assert!(html.contains("embedded web UI"));
    }

    #[test]
    fn build_preview_reuses_inventory_preview_source() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        let leaf = repo.path().join(".leaf/02-leaves/demo");
        fs::create_dir_all(leaf.join("01-Learn")).expect("learn dir");
        fs::write(
            leaf.join("00-status.md"),
            "# 상태\n\n- stage: leaf\n- current phase: Learn\n- next action: 다음 행동\n",
        )
        .expect("status");
        fs::write(
            leaf.join("01-Learn/01-intent.md"),
            "# 의도\n\n미리보기 본문\n",
        )
        .expect("intent");

        let preview = build_preview(repo.path(), "demo").expect("preview");

        assert_eq!(preview.title, "demo");
        assert!(preview.lines.iter().any(|line| {
            matches!(
                line,
                crate::preview::PreviewLineJson::SourceBoundary {
                    phase,
                    gate,
                    ..
                } if phase == "Learn" && gate == "① Intent"
            )
        }));
        assert!(preview.lines.iter().any(|line| {
            matches!(
                line,
                crate::preview::PreviewLineJson::Text { text } if text == "미리보기 본문"
            )
        }));
    }
}
