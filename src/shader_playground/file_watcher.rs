use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use log::debug;
use notify_debouncer_mini::{
    new_debouncer, notify::RecommendedWatcher, DebounceEventResult, Debouncer,
};

pub struct FileWatcher {
    debouncer: Debouncer<RecommendedWatcher>,
    watching: HashMap<PathBuf, HashSet<OsString>>,
    watcher_rx: std::sync::mpsc::Receiver<DebounceEventResult>,
}

impl FileWatcher {
    pub fn new(debouncer_ms: u32) -> anyhow::Result<Self> {
        let (watcher_tx, watcher_rx) = std::sync::mpsc::channel();

        let debouncer = new_debouncer(Duration::from_millis(debouncer_ms as _), watcher_tx)
            .with_context(|| "setting up file watcher")?;

        Ok(Self {
            debouncer,
            watching: HashMap::new(),
            watcher_rx,
        })
    }

    fn prepare_path(path: &Path) -> Option<(PathBuf, OsString)> {
        let path = path.canonicalize().ok()?;
        let parent = path.parent().map(|p| p.to_path_buf());
        let name = path.file_name().map(|s| s.to_owned());
        parent.zip(name)
    }

    pub fn file_changed(&self) -> Option<PathBuf> {
        let mut out = None;
        while let Ok(Ok(events)) = self.watcher_rx.try_recv() {
            for event in events {
                let path = event.path;
                if let Some((dir, file)) = Self::prepare_path(&path) {
                    let has_matched_entry = self.watching.iter().any(|(i_dir, files)| {
                        if dir == *i_dir {
                            files.iter().any(|i_file| *i_file == *file)
                        } else {
                            false
                        }
                    });
                    if has_matched_entry {
                        out = Some(path);
                    }
                }
            }
        }
        out
    }

    pub fn watch(&mut self, path: &Path) -> anyhow::Result<()> {
        if let Some((dir, file)) = Self::prepare_path(path) {
            let watch_new_dir = !self.watching.contains_key(&dir);
            self.watching.entry(dir.clone()).or_default().insert(file);

            if watch_new_dir {
                debug!("watching dir `{}`", dir.display());
                self.debouncer.watcher().watch(
                    &dir,
                    notify_debouncer_mini::notify::RecursiveMode::NonRecursive,
                )?;
            }
        }
        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> anyhow::Result<()> {
        debug!("unwatching {}", path.display());
        if let Some((dir, file)) = Self::prepare_path(path) {
            let remove_dir = if let Some(entry) = self.watching.get_mut(&dir) {
                entry.remove(&file);
                entry.is_empty()
            } else {
                false
            };

            if remove_dir {
                self.watching.remove(&dir);
                self.debouncer.watcher().unwatch(path)?;
            }
        }

        Ok(())
    }

    pub fn unwatch_all(&mut self) -> anyhow::Result<()> {
        for (dir, files) in self.watching.drain() {
            debug!(
                "unwatching dir `{}` which contains `{:?}`",
                dir.display(),
                files
            );
            self.debouncer.watcher().unwatch(&dir)?;
        }
        Ok(())
    }
}
