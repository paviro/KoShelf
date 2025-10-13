use crate::site_generator::SiteGenerator;
use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use log::{info, warn, debug};

pub struct FileWatcher {
    books_path: Option<PathBuf>,
    site_dir: PathBuf,
    site_title: String,
    include_unread: bool,
    statistics_db_path: Option<PathBuf>,
    heatmap_scale_max: Option<u32>,
    rebuild_tx: Option<mpsc::UnboundedSender<()>>,
}

impl FileWatcher {
    pub async fn new(
        books_path: Option<PathBuf>,
        site_dir: PathBuf,
        site_title: String,
        include_unread: bool,
        statistics_db_path: Option<PathBuf>,
        heatmap_scale_max: Option<u32>,
    ) -> Result<Self> {
        Ok(Self {
            books_path,
            site_dir,
            site_title,
            include_unread,
            statistics_db_path,
            heatmap_scale_max,
            rebuild_tx: None,
        })
    }
    
    pub async fn run(mut self) -> Result<()> {
        let (file_tx, mut file_rx) = mpsc::unbounded_channel();
        let (rebuild_tx, mut rebuild_rx) = mpsc::unbounded_channel::<()>();
        self.rebuild_tx = Some(rebuild_tx);
        
        // Set up file watcher
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                match result {
                    Ok(event) => {
                        if let Err(e) = file_tx.send(event) {
                            warn!("Failed to send file event: {}", e);
                        }
                    }
                    Err(e) => warn!("File watcher error: {}", e),
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;
        
        // Watch the books path if provided
        if let Some(ref books_path) = self.books_path {
            watcher.watch(books_path, RecursiveMode::Recursive)?;
            info!("File watcher started for books directory: {:?}", books_path);
        }
        
        // Also watch the statistics database if provided
        if let Some(ref stats_path) = self.statistics_db_path {
            if stats_path.exists() {
                // Watch the parent directory of the statistics database
                if let Some(parent) = stats_path.parent() {
                    watcher.watch(parent, RecursiveMode::NonRecursive)?;
                    info!("File watcher started for statistics database: {:?}", stats_path);
                }
            }
        }
        
        // Clone necessary data for the rebuild task
        let books_path_clone = self.books_path.clone();
        let site_dir_clone = self.site_dir.clone();
        let site_title_clone = self.site_title.clone();
        let include_unread_clone = self.include_unread;
        let statistics_db_path_clone = self.statistics_db_path.clone();
        let heatmap_scale_max_clone = self.heatmap_scale_max;
        
        // Spawn delayed rebuild task
        let rebuild_task = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let rebuild_delay = Duration::from_secs(10); // Wait 10 seconds after last event
                
                while let Some(_) = rebuild_rx.recv().await {
                    // Wait for the delay period to debounce multiple events
                    sleep(rebuild_delay).await;
                    
                    // Drain any additional events that came in during the delay
                    while rebuild_rx.try_recv().is_ok() {}
                    
                    info!("Starting delayed site rebuild after quiet period");
                    
                    // Create new site generator and regenerate everything
                    let site_generator = SiteGenerator::new(
                        site_dir_clone.clone(),
                        site_title_clone.clone(),
                        include_unread_clone,
                        books_path_clone.clone(),
                        statistics_db_path_clone.clone(),
                        heatmap_scale_max_clone,
                    );
                    
                    match site_generator.generate().await {
                        Ok(_) => info!("Delayed site rebuild completed successfully"),
                        Err(e) => warn!("Failed to rebuild site: {}", e),
                    }
                }
            })
        });
        
        // Main file event processing loop
        while let Some(event) = file_rx.recv().await {
            if self.should_process_event(&event) {

                // Log what triggered the rebuild
                self.log_file_event(&event);
                
                // Just queue a rebuild - no need to track individual file changes anymore
                if let Some(ref tx) = self.rebuild_tx {
                    let _ = tx.send(());
                }
            }
        }
        
        rebuild_task.abort();
        Ok(())
    }
    
    fn should_process_event(&self, event: &Event) -> bool {
        match &event.kind {
            // Only process actual file changes, not access events
            EventKind::Create(_) | EventKind::Remove(_) => {
                self.event_affects_relevant_files(event)
            }
            EventKind::Modify(modify_kind) => {
                // Only process content modifications, not metadata or access
                use notify::event::ModifyKind;
                match modify_kind {
                    ModifyKind::Data(_) => self.event_affects_relevant_files(event),
                    ModifyKind::Name(_) => true, // Renames are important
                    ModifyKind::Any => {
                        // For Any modifications, we need to be more careful
                        // Log these for debugging but still process them
                        debug!("Processing Modify(Any) event: {:?}", event);
                        self.event_affects_relevant_files(event)
                    }
                    _ => {
                        debug!("Ignoring modify event: {:?}", modify_kind);
                        false
                    }
                }
            }
            _ => {
                debug!("Ignoring event kind: {:?}", event.kind);
                false
            }
        }
    }
    
    fn event_affects_relevant_files(&self, event: &Event) -> bool {
        event.paths.iter().any(|path| {
		let extension = path
			.extension()
			.and_then(|s| s.to_str())
			.map(|ext| ext.to_ascii_lowercase());
            let filename = path.file_name().and_then(|s| s.to_str());
            
            // Check for EPUB files and metadata files
		if extension.as_deref() == Some("epub") || filename == Some("metadata.epub.lua") {
                return true;
            }
            
            // Check for .sdr directories (KoReader metadata folders)
            if let Some(filename) = filename {
                if filename.ends_with(".sdr") {
                    return true;
                }
            }
            
            // Check for statistics database files
            if let Some(ref stats_path) = self.statistics_db_path {
                if path == stats_path {
                    return true;
                }
            }
            
            false
        })
    }
    
    fn log_file_event(&self, event: &Event) {
        for path in &event.paths {
            let filename = path.file_name().and_then(|s| s.to_str());
            
            match &event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
					// Check EPUB files
					if path
						.extension()
						.and_then(|s| s.to_str())
						.map(|ext| ext.eq_ignore_ascii_case("epub"))
						.unwrap_or(false)
					{
                        info!("EPUB file modified: {:?}", path);
                    }
                    
                    // Check metadata files
                    if filename == Some("metadata.epub.lua") {
                        info!("Metadata file modified: {:?}", path);
                    }
                    
                    // Check .sdr directories
                    if let Some(filename) = filename {
                        if filename.ends_with(".sdr") {
                            info!("KoReader metadata directory modified: {:?}", path);
                        }
                    }
                    
                    // Check statistics database
                    if let Some(ref stats_path) = self.statistics_db_path {
                        if path == stats_path {
                            info!("Statistics database modified: {:?}", path);
                        }
                    }
                }
                EventKind::Remove(_) => {
					// Check EPUB files
					if path
						.extension()
						.and_then(|s| s.to_str())
						.map(|ext| ext.eq_ignore_ascii_case("epub"))
						.unwrap_or(false)
					{
                        info!("EPUB file removed: {:?}", path);
                    }
                    
                    // Check metadata files
                    if filename == Some("metadata.epub.lua") {
                        info!("Metadata file removed: {:?}", path);
                    }
                    
                    // Check .sdr directories  
                    if let Some(filename) = filename {
                        if filename.ends_with(".sdr") {
                            info!("KoReader metadata directory removed: {:?}", path);
                        }
                    }
                    
                    // Check statistics database
                    if let Some(ref stats_path) = self.statistics_db_path {
                        if path == stats_path {
                            info!("Statistics database removed: {:?}", path);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
} 