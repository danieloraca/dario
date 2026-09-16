//! The save format shared by the desktop game and the web server.
use serde::{Deserialize, Serialize};

pub const LEVEL_COUNT: usize = 16;
pub const MAX_SAVE_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub cleared: bool,
    pub speed_medal: bool,
    pub treasure_medal: bool,
    pub best_ms: Option<u64>,
    pub high_score: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub version: u32,
    pub levels: [Record; LEVEL_COUNT],
    pub arcade_best: u32,
    pub muted: bool,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            version: 1,
            levels: std::array::from_fn(|_| Record::default()),
            arcade_best: 0,
            muted: false,
        }
    }
}

impl Progress {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_SAVE_BYTES {
            return Err("Save is too large".into());
        }
        let save: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        if save.version != 1 {
            return Err(format!("Unsupported save version {}", save.version));
        }
        Ok(save)
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).expect("save consists only of JSON values")
    }

    /// The first uncleared level is the next campaign destination.
    pub fn unlocked(&self) -> usize {
        self.levels
            .iter()
            .take_while(|record| record.cleared)
            .count()
            .min(LEVEL_COUNT - 1)
    }

    pub fn finish(&mut self, stage: usize, millis: u64, score: u32, speed: bool, treasure: bool) {
        let record = &mut self.levels[stage];
        record.cleared = true;
        record.speed_medal |= speed;
        record.treasure_medal |= treasure;
        record.best_ms = Some(record.best_ms.map_or(millis, |best| best.min(millis)));
        record.high_score = record.high_score.max(score);
    }

    /// Merge snapshots from different tabs without losing earned progress.
    pub fn merge(&mut self, incoming: &Self) {
        for (record, newer) in self.levels.iter_mut().zip(&incoming.levels) {
            record.cleared |= newer.cleared;
            record.speed_medal |= newer.speed_medal;
            record.treasure_medal |= newer.treasure_medal;
            record.best_ms = match (record.best_ms, newer.best_ms) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            record.high_score = record.high_score.max(newer.high_score);
        }
        self.arcade_best = self.arcade_best.max(incoming.arcade_best);
        self.muted = incoming.muted;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod file {
    use super::Progress;
    use std::{
        fs,
        io::{self, Write},
        path::Path,
    };

    /// Missing saves are new profiles. Other failures must be shown to the player.
    pub fn load(path: &Path) -> Result<Progress, String> {
        match fs::read(path) {
            Ok(bytes) => Progress::decode(&bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Progress::default()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn save(path: &Path, progress: &Progress) -> io::Result<()> {
        let directory = path
            .parent()
            .ok_or_else(|| io::Error::other("Save needs a parent directory"))?;
        fs::create_dir_all(directory)?;
        // Keep the temporary file on the same filesystem; a failed write leaves
        // the previous JSON intact. NamedTempFile also avoids shared temp names.
        let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
        temporary.write_all(&progress.encode())?;
        temporary.as_file().sync_all()?;
        temporary.persist(path).map_err(|error| error.error)?;
        #[cfg(unix)]
        fs::File::open(directory)?.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_accumulate_across_separate_attempts_and_stale_snapshots() {
        let mut save = Progress::default();
        save.finish(0, 40_000, 3100, true, false);
        let mut other_tab = Progress::default();
        other_tab.finish(0, 62_000, 5600, false, true);
        other_tab.finish(1, 75_000, 2100, false, false);
        save.merge(&other_tab);
        assert_eq!(save.unlocked(), 2);
        assert_eq!(
            save.levels[0],
            Record {
                cleared: true,
                speed_medal: true,
                treasure_medal: true,
                best_ms: Some(40_000),
                high_score: 5600,
            }
        );
        save.merge(&Progress::default());
        assert_eq!(save.unlocked(), 2);
        assert!(save.levels[0].treasure_medal);
    }

    #[test]
    fn decoding_rejects_damaged_or_future_saves() {
        let save = Progress::default();
        assert_eq!(Progress::decode(&save.encode()).unwrap(), save);
        assert!(Progress::decode(b"{broken").is_err());
        let mut future = save;
        future.version = 2;
        assert!(Progress::decode(&future.encode()).is_err());
        assert!(Progress::decode(&vec![b' '; MAX_SAVE_BYTES + 1]).is_err());
    }

    #[test]
    fn atomic_save_round_trips_and_corruption_is_not_a_fresh_profile() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("players/alex.json");
        assert_eq!(file::load(&path).unwrap(), Progress::default());
        let mut save = Progress::default();
        save.finish(0, 31_250, 4300, true, true);
        file::save(&path, &save).unwrap();
        save.finish(1, 55_125, 7200, false, true);
        file::save(&path, &save).unwrap();
        assert_eq!(file::load(&path).unwrap(), save);
        std::fs::write(&path, b"interrupted old save").unwrap();
        assert!(file::load(&path).is_err());
    }
}
