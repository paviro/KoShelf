use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use include_dir::{Dir, include_dir};

use crate::pipeline::embed::is_precompressed;

static FRONTEND_DIST: Dir = include_dir!("$OUT_DIR/frontend_dist");

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/", get(react_shell_index_handler))
        .route("/index.html", get(react_shell_index_handler))
        .route("/core/{*path}", get(react_shell_core_asset_handler))
        .route("/{path}", get(react_shell_root_asset_handler))
}

async fn react_shell_index_handler() -> Response {
    serve_embedded_frontend_file("index.html")
}

async fn react_shell_root_asset_handler(Path(path): Path<String>) -> Response {
    serve_embedded_frontend_file(&path)
}

async fn react_shell_core_asset_handler(Path(path): Path<String>) -> Response {
    let full_path = format!("core/{}", path);
    serve_embedded_frontend_file(&full_path)
}

fn serve_embedded_frontend_file(path: &str) -> Response {
    if path.contains("..") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let Some(file) = FRONTEND_DIST.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if is_precompressed(path) {
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, guess_content_type(path)),
                (header::CONTENT_ENCODING, "gzip"),
            ],
            file.contents(),
        )
            .into_response()
    } else {
        (
            StatusCode::OK,
            [(CONTENT_TYPE, guess_content_type(path))],
            file.contents(),
        )
            .into_response()
    }
}

fn guess_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") || path.ends_with(".mjs") {
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
