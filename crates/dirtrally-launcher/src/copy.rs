//! Making the private copy in the background, with progress for the
//! settings window (the first copy of the Linux port is about 40 GB).
//!
//! [`CopyJob::start`] runs [`runtime::prepare`] on a worker thread. A
//! second thread measures how much of the game folder is in the runtime
//! already (the sizes of the files [`runtime::list`] finds in the game
//! folder, counted when the runtime holds them at full size), so the window
//! can show "12 of 40 GB" without the SDK's sync reporting anything itself.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rea_launcher::paths;
use rea_launcher::runtime as sdk_runtime;

use crate::runtime;

/// Where a copy job is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Bytes already in the runtime, and the game folder's total.
    Copying {
        done: u64,
        total: u64,
    },
    /// Done: how many files were brought over.
    Done {
        brought: usize,
    },
    Failed(String),
}

impl State {
    /// The share done, 0..=1.
    pub fn fraction(&self) -> f32 {
        match self {
            State::Copying { done, total } if *total > 0 => {
                (*done as f64 / *total as f64).min(1.0) as f32
            }
            State::Copying { .. } => 0.0,
            State::Done { .. } => 1.0,
            State::Failed(_) => 0.0,
        }
    }
}

/// "12.4 of 40.1 GB" (one decimal, decimal gigabytes, as file managers show).
pub fn amount(done: u64, total: u64) -> String {
    let gb = |bytes: u64| bytes as f64 / 1e9;
    format!("{:.1} of {:.1} GB", gb(done), gb(total))
}

/// How many bytes of `listing` (the game folder's files) the runtime holds
/// at their full size.
pub fn bytes_present(runtime: &Path, listing: &sdk_runtime::Listing) -> u64 {
    listing
        .iter()
        .filter(|(rel, stamp)| {
            std::fs::metadata(paths::native(runtime, rel)).is_ok_and(|m| m.len() == stamp.size)
        })
        .map(|(_, stamp)| stamp.size)
        .sum()
}

/// A copy running in the background.
pub struct CopyJob {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
}

impl CopyJob {
    /// Starts bringing `game` into `runtime`.
    pub fn start(game: PathBuf, runtime_dir: PathBuf) -> CopyJob {
        let state = Arc::new(Mutex::new(State::Copying { done: 0, total: 0 }));
        let stop = Arc::new(AtomicBool::new(false));
        {
            let (state, stop, game, rt) = (
                state.clone(),
                stop.clone(),
                game.clone(),
                runtime_dir.clone(),
            );
            thread::spawn(move || {
                let Ok(listing) = sdk_runtime::list(&game) else {
                    return;
                };
                let total: u64 = listing.values().map(|stamp| stamp.size).sum();
                while !stop.load(Ordering::Relaxed) {
                    let done = bytes_present(&rt, &listing);
                    let mut now = state.lock().unwrap_or_else(|e| e.into_inner());
                    if matches!(*now, State::Copying { .. }) {
                        *now = State::Copying { done, total };
                    } else {
                        break;
                    }
                    drop(now);
                    thread::sleep(Duration::from_millis(500));
                }
            });
        }
        {
            let state = state.clone();
            thread::spawn(move || {
                let result = runtime::prepare(&game, &runtime_dir);
                let mut now = state.lock().unwrap_or_else(|e| e.into_inner());
                *now = match result {
                    Ok(report) => State::Done {
                        brought: report.brought,
                    },
                    Err(error) => State::Failed(error),
                };
            });
        }
        CopyJob { state, stop }
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Drop for CopyJob {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn amounts_read_like_a_file_manager() {
        assert_eq!(amount(12_400_000_000, 40_100_000_000), "12.4 of 40.1 GB");
        assert_eq!(amount(0, 0), "0.0 of 0.0 GB");
        assert_eq!(State::Copying { done: 5, total: 10 }.fraction(), 0.5);
        assert_eq!(State::Copying { done: 0, total: 0 }.fraction(), 0.0);
        assert_eq!(State::Done { brought: 1 }.fraction(), 1.0);
    }

    #[test]
    fn the_copy_finishes_and_only_whole_files_count_as_present() {
        let root = std::env::temp_dir().join(format!("dirtrally-copy-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (game, rt) = (root.join("Jogos d'Ação"), root.join("runtime"));
        fs::create_dir_all(game.join("bin")).unwrap();
        fs::create_dir_all(game.join("share/data")).unwrap();
        fs::write(game.join("bin/DirtRally"), vec![1u8; 300]).unwrap();
        fs::write(game.join("share/data/game.bin"), vec![2u8; 700]).unwrap();
        let listing = sdk_runtime::list(&game).unwrap();
        assert_eq!(bytes_present(&rt, &listing), 0);
        // A half-written file does not count (a copy that was stopped).
        fs::create_dir_all(rt.join("bin")).unwrap();
        fs::write(rt.join(runtime::OWNED_MARKER), b"").unwrap();
        fs::write(rt.join("bin/DirtRally"), vec![1u8; 100]).unwrap();
        assert_eq!(bytes_present(&rt, &listing), 0);

        let job = CopyJob::start(game.clone(), rt.clone());
        let mut state = job.state();
        for _ in 0..200 {
            if !matches!(state, State::Copying { .. }) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
            state = job.state();
        }
        assert!(matches!(state, State::Done { .. }), "{state:?}");
        assert_eq!(bytes_present(&rt, &listing), 1000);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_runtime_inside_the_game_folder_fails_with_the_reason() {
        let root = std::env::temp_dir().join(format!("dirtrally-copy-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).unwrap();
        let job = CopyJob::start(root.clone(), root.join("runtime"));
        let mut state = job.state();
        for _ in 0..200 {
            if !matches!(state, State::Copying { .. }) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
            state = job.state();
        }
        assert!(matches!(state, State::Failed(_)), "{state:?}");
        fs::remove_dir_all(&root).unwrap();
    }
}
