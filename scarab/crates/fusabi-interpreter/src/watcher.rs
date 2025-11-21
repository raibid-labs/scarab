use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use crate::error::{FusabiError, Result};

/// File watcher for hot-reloading scripts
pub struct ScriptWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watched_paths: Vec<PathBuf>,
}

impl ScriptWatcher {
    /// Create a new script watcher
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            notify::Config::default()
                .with_poll_interval(Duration::from_millis(100)),
        )
        .map_err(|e| FusabiError::WatcherError(e.to_string()))?;

        Ok(Self {
            watcher,
            receiver: rx,
            watched_paths: Vec::new(),
        })
    }

    /// Watch a file or directory for changes
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(|e| FusabiError::WatcherError(e.to_string()))?;
        self.watched_paths.push(path.to_path_buf());
        Ok(())
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.watcher
            .unwatch(path)
            .map_err(|e| FusabiError::WatcherError(e.to_string()))?;
        self.watched_paths.retain(|p| p != path);
        Ok(())
    }

    /// Poll for file change events (non-blocking)
    pub fn poll_events(&self) -> Vec<FileChangeEvent> {
        let mut events = Vec::new();

        // Drain all pending events
        while let Ok(result) = self.receiver.try_recv() {
            match result {
                Ok(event) => {
                    // Filter for .fsx files
                    let paths: Vec<_> = event
                        .paths
                        .into_iter()
                        .filter(|p| {
                            p.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e == "fsx")
                                .unwrap_or(false)
                        })
                        .collect();

                    if !paths.is_empty() {
                        events.push(FileChangeEvent {
                            kind: event.kind,
                            paths,
                        });
                    }
                }
                Err(e) => {
                    eprintln!("File watcher error: {:?}", e);
                }
            }
        }

        events
    }

    /// Get list of watched paths
    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

/// File change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub kind: notify::EventKind,
    pub paths: Vec<PathBuf>,
}

impl FileChangeEvent {
    /// Check if this is a modification event
    pub fn is_modify(&self) -> bool {
        matches!(self.kind, notify::EventKind::Modify(_))
    }

    /// Check if this is a create event
    pub fn is_create(&self) -> bool {
        matches!(self.kind, notify::EventKind::Create(_))
    }

    /// Check if this is a remove event
    pub fn is_remove(&self) -> bool {
        matches!(self.kind, notify::EventKind::Remove(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    #[ignore] // Requires filesystem access
    fn test_watcher_basic() {
        let temp_dir = std::env::temp_dir().join("fusabi_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let mut watcher = ScriptWatcher::new().unwrap();
        watcher.watch(&temp_dir).unwrap();

        // Create a test file
        let test_file = temp_dir.join("test.fsx");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"let x = 42").unwrap();

        // Give the watcher time to detect the change
        std::thread::sleep(Duration::from_millis(200));

        let events = watcher.poll_events();
        assert!(!events.is_empty());

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
