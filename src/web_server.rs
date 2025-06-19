use anyhow::Result;
use axum::Router;
use log::info;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::CorsLayer;

pub struct WebServer {
    site_dir: PathBuf,
    port: u16,
}

impl WebServer {
    pub fn new(site_dir: PathBuf, port: u16) -> Self {
        Self { site_dir, port }
    }
    
    pub async fn run(self) -> Result<()> {
        // Create a 404 page service
        let not_found_service = ServeFile::new("templates/404.html");
        
        let app = Router::new()
            .fallback_service(ServeDir::new(&self.site_dir)
                .not_found_service(not_found_service))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            );
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        
        info!("Web server running on http://localhost:{}, binding to: 0.0.0.0", self.port);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
} 