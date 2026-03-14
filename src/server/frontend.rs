use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};

static FRONTEND_DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

pub fn routes() -> Router {
    Router::new()
        .route("/", get(react_shell_index_handler))
        .route("/index.html", get(react_shell_index_handler))
        .route("/manifest.json", get(react_shell_manifest_handler))
        .route("/assets/icons/{*path}", get(react_shell_icon_handler))
        .route("/assets/js/{*path}", get(react_shell_js_handler))
        .route("/assets/css/{*path}", get(react_shell_css_handler))
}

async fn react_shell_index_handler() -> Response {
    serve_embedded_frontend_file("index.html")
}

async fn react_shell_manifest_handler() -> Response {
    serve_embedded_frontend_file("manifest.json")
}

async fn react_shell_icon_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/icons/{}", path);
    serve_embedded_frontend_file(&full_path)
}

async fn react_shell_js_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/js/{}", path);
    serve_embedded_frontend_file(&full_path)
}

async fn react_shell_css_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("assets/css/{}", path);
    serve_embedded_frontend_file(&full_path)
}

fn serve_embedded_frontend_file(path: &str) -> Response {
    let Some(file) = FRONTEND_DIST.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        StatusCode::OK,
        [(CONTENT_TYPE, guess_content_type(path))],
        file.contents(),
    )
        .into_response()
}

fn guess_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".map") {
        "application/json; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}
