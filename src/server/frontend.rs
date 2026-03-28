use std::io::Read as _;

use axum::{
    Router,
    extract::Path,
    http::{HeaderMap, StatusCode, header, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::get,
};
use flate2::read::GzDecoder;
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

async fn react_shell_index_handler(headers: HeaderMap) -> Response {
    serve_embedded_frontend_file("index.html", &headers)
}

async fn react_shell_root_asset_handler(headers: HeaderMap, Path(path): Path<String>) -> Response {
    serve_embedded_frontend_file(&path, &headers)
}

async fn react_shell_core_asset_handler(headers: HeaderMap, Path(path): Path<String>) -> Response {
    let full_path = format!("core/{}", path);
    serve_embedded_frontend_file(&full_path, &headers)
}

fn serve_embedded_frontend_file(path: &str, headers: &HeaderMap) -> Response {
    if path.contains("..") {
        return StatusCode::NOT_FOUND.into_response();
    }

    let Some(file) = FRONTEND_DIST.get_file(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if is_precompressed(path) {
        if accepts_gzip(headers) {
            (
                StatusCode::OK,
                [
                    (CONTENT_TYPE, guess_content_type(path)),
                    (header::CONTENT_ENCODING, "gzip"),
                ],
                file.contents().to_vec(),
            )
                .into_response()
        } else {
            let mut decoder = GzDecoder::new(file.contents());
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_err() {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            (
                StatusCode::OK,
                [(CONTENT_TYPE, guess_content_type(path))],
                decompressed,
            )
                .into_response()
        }
    } else {
        (
            StatusCode::OK,
            [(CONTENT_TYPE, guess_content_type(path))],
            file.contents(),
        )
            .into_response()
    }
}

fn accepts_gzip(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("gzip"))
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
