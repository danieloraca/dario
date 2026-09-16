//! Native save lifecycle. A read failure disables writes until the next launch.
use dario_progress::Progress;
use std::{env, path::PathBuf};

pub struct Storage {
    path: Option<PathBuf>,
    saved: Progress,
    retry_at: f64,
    pub notice: Option<&'static str>,
}

fn save_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("DARIO_SAVE_PATH") {
        return env::current_dir()
            .map(|base| base.join(path))
            .map_err(|error| error.to_string());
    }
    #[cfg(target_os = "macos")]
    let directory = env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/Dario"));
    #[cfg(target_os = "windows")]
    let directory = env::var_os("APPDATA").map(|home| PathBuf::from(home).join("Dario"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let directory = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .map(|base| base.join("dario"));
    directory
        .map(|path| path.join("progress.json"))
        .ok_or_else(|| "Cannot find the app data folder".into())
}

impl Storage {
    pub fn load(smoke: bool) -> (Self, Progress) {
        if smoke {
            return (
                Self {
                    path: None,
                    saved: Progress::default(),
                    retry_at: 0.0,
                    notice: None,
                },
                Progress::default(),
            );
        }
        let result = save_path()
            .and_then(|path| dario_progress::file::load(&path).map(|progress| (path, progress)));
        match result {
            Ok((path, progress)) => (
                Self {
                    path: Some(path),
                    saved: progress.clone(),
                    retry_at: 0.0,
                    notice: None,
                },
                progress,
            ),
            Err(error) => {
                eprintln!("Progress could not be loaded; existing save will be preserved: {error}");
                (
                    Self {
                        path: None,
                        saved: Progress::default(),
                        retry_at: 0.0,
                        notice: Some("SAVE UNAVAILABLE - PLAYING AS GUEST"),
                    },
                    Progress::default(),
                )
            }
        }
    }

    pub fn sync(&mut self, progress: &Progress, now: f64) {
        let Some(path) = &self.path else {
            return;
        };
        if *progress == self.saved || now < self.retry_at {
            return;
        }
        match dario_progress::file::save(path, progress) {
            Ok(()) => {
                self.saved = progress.clone();
                self.notice = None;
            }
            Err(error) => {
                eprintln!("Could not save progress to {}: {error}", path.display());
                self.retry_at = now + 5.0;
                self.notice = Some("SAVE FAILED - RETRYING");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autosave_persists_changes_and_smoke_runs_never_write() {
        let path = env::temp_dir().join(format!("dario-autosave-{}.json", std::process::id()));
        let mut storage = Storage {
            path: Some(path.clone()),
            saved: Progress::default(),
            retry_at: 0.0,
            notice: None,
        };
        let mut progress = Progress::default();
        progress.levels[0].cleared = true;
        progress.muted = true;
        storage.sync(&progress, 1.0);
        assert_eq!(dario_progress::file::load(&path).unwrap(), progress);
        assert!(storage.notice.is_none());
        std::fs::remove_file(path).unwrap();
        let (mut smoke, _) = Storage::load(true);
        smoke.sync(&progress, 1.0);
        assert!(smoke.path.is_none());
    }
}
