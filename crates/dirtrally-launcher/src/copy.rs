//! Making the private copy in the background, with progress for the
//! settings window (the first copy of the Linux port is about 40 GB).
//!
//! [`CopyJob::start`] runs [`runtime::prepare_with_progress`] on a worker
//! thread; the SDK's sync reports each file it brings over into a
//! [`ProgressCounter`], so the window can show "12 of 40 GB" of what this
//! copy brings (all of it the first time, only what changed later).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use rea_launcher::runtime::{Progress, ProgressCounter};

use crate::runtime;

/// Where a copy job is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Bytes brought over so far, and what this copy brings in all (0
    /// until the game folder is listed).
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

/// A job's state: its result once there is one, else the counter's bytes.
pub fn state_of(counter: &ProgressCounter, result: Option<State>) -> State {
    result.unwrap_or_else(|| {
        let now = counter.get();
        State::Copying {
            done: now.bytes_done,
            total: now.bytes_total,
        }
    })
}

/// A copy running in the background.
pub struct CopyJob {
    counter: ProgressCounter,
    result: Arc<Mutex<Option<State>>>,
}

impl CopyJob {
    /// Starts bringing `game` into `runtime`.
    pub fn start(game: PathBuf, runtime_dir: PathBuf) -> CopyJob {
        CopyJob::start_with(game, runtime_dir, ProgressCounter::new())
    }

    /// [`CopyJob::start`], reporting into `counter` (tests read it).
    pub fn start_with(game: PathBuf, runtime_dir: PathBuf, counter: ProgressCounter) -> CopyJob {
        let result = Arc::new(Mutex::new(None));
        {
            let (counter, result) = (counter.clone(), result.clone());
            thread::spawn(move || {
                let report =
                    runtime::prepare_with_progress(&game, &runtime_dir, &mut |p: Progress| {
                        counter.set(p)
                    });
                *result.lock().unwrap_or_else(|e| e.into_inner()) = Some(match report {
                    Ok(report) => State::Done {
                        brought: report.brought,
                    },
                    Err(error) => State::Failed(error),
                });
            });
        }
        CopyJob { counter, result }
    }

    pub fn state(&self) -> State {
        let result = self
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        state_of(&self.counter, result)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    #[test]
    fn amounts_read_like_a_file_manager() {
        assert_eq!(amount(12_400_000_000, 40_100_000_000), "12.4 of 40.1 GB");
        assert_eq!(amount(0, 0), "0.0 of 0.0 GB");
        assert_eq!(State::Copying { done: 5, total: 10 }.fraction(), 0.5);
        assert_eq!(State::Copying { done: 0, total: 0 }.fraction(), 0.0);
        assert_eq!(State::Done { brought: 1 }.fraction(), 1.0);
    }

    fn wait(job: &CopyJob) -> State {
        let mut state = job.state();
        for _ in 0..200 {
            if !matches!(state, State::Copying { .. }) {
                break;
            }
            thread::sleep(Duration::from_millis(20));
            state = job.state();
        }
        state
    }

    /// The bytes come from the SDK's sync as it brings each file over: a
    /// half-written file (a copy that was stopped) is brought again.
    #[test]
    fn the_copy_reports_what_it_brings_and_finishes() {
        let root = std::env::temp_dir().join(format!("dirtrally-copy-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (game, rt) = (root.join("Jogos d'Ação"), root.join("runtime"));
        fs::create_dir_all(game.join("bin")).unwrap();
        fs::create_dir_all(game.join("share/data")).unwrap();
        fs::write(game.join("bin/DirtRally"), vec![1u8; 300]).unwrap();
        fs::write(game.join("share/data/game.bin"), vec![2u8; 700]).unwrap();
        fs::create_dir_all(rt.join("bin")).unwrap();
        fs::write(rt.join(runtime::OWNED_MARKER), b"").unwrap();
        fs::write(rt.join("bin/DirtRally"), vec![1u8; 100]).unwrap();

        let counter = ProgressCounter::new();
        let job = CopyJob::start_with(game.clone(), rt.clone(), counter.clone());
        let state = wait(&job);
        assert!(matches!(state, State::Done { brought: 2 }), "{state:?}");
        let last = counter.get();
        assert_eq!((last.bytes_done, last.bytes_total), (1000, 1000));
        assert_eq!(fs::metadata(rt.join("bin/DirtRally")).unwrap().len(), 300);

        // Up to date: nothing to bring, done at once.
        let job = CopyJob::start(game.clone(), rt.clone());
        assert!(matches!(wait(&job), State::Done { brought: 0 }));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn while_copying_the_state_is_the_counters_bytes() {
        let counter = ProgressCounter::new();
        counter.set(Progress {
            files_done: 1,
            files_total: 4,
            bytes_done: 12,
            bytes_total: 40,
        });
        assert_eq!(
            state_of(&counter, None),
            State::Copying {
                done: 12,
                total: 40
            }
        );
        let failed = State::Failed("disk full".into());
        assert_eq!(state_of(&counter, Some(failed.clone())), failed);
    }

    #[test]
    fn a_runtime_inside_the_game_folder_fails_with_the_reason() {
        let root = std::env::temp_dir().join(format!("dirtrally-copy-bad-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).unwrap();
        let job = CopyJob::start(root.clone(), root.join("runtime"));
        let state = wait(&job);
        assert!(matches!(state, State::Failed(_)), "{state:?}");
        fs::remove_dir_all(&root).unwrap();
    }
}
