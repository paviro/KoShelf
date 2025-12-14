//! Shared state for version update notifications between file watcher and web server.

use std::sync::Arc;
use tokio::sync::broadcast;

/// Shared state for notifying clients when the site has been rebuilt.
#[derive(Clone)]
pub struct VersionNotifier {
    /// Broadcast channel sender - sends the new version string when site is rebuilt
    sender: broadcast::Sender<String>,
}

impl VersionNotifier {
    /// Create a new version notifier.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(16);
        Self { sender }
    }

    /// Notify all waiting clients that a new version is available.
    pub fn notify(&self, version: String) {
        // Ignore errors (no receivers is OK)
        let _ = self.sender.send(version);
    }

    /// Subscribe to version update notifications.
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}

impl Default for VersionNotifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Arc-wrapped version notifier for sharing across tasks.
pub type SharedVersionNotifier = Arc<VersionNotifier>;

/// Create a new shared version notifier.
pub fn create_version_notifier() -> SharedVersionNotifier {
    Arc::new(VersionNotifier::new())
}
