//! The library against the real kernel: an execute-only page is readable
//! with it preloaded. Without it, the same read faults on CPUs with memory
//! protection keys (the DiRT Rally crash), which proves the test can fail.
#![cfg(target_os = "linux")]

use std::path::PathBuf;
use std::process::Command;

fn probe() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_exec-read-probe"));
    command.env_remove("LD_PRELOAD");
    command
}

/// The cdylib cargo builds beside the probe (or in its `deps/` folder when
/// it was built for the tests).
fn library() -> PathBuf {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_exec-read-probe"));
    let dir = bin.parent().expect("the probe has a folder");
    [dir.to_path_buf(), dir.join("deps")]
        .into_iter()
        .map(|dir| dir.join("libdirtrally_exec_read.so"))
        .find(|lib| lib.is_file())
        .unwrap_or_else(|| dir.join("libdirtrally_exec_read.so"))
}

fn has_pku() -> bool {
    std::fs::read_to_string("/proc/cpuinfo")
        .map(|info| {
            info.lines()
                .filter(|line| line.starts_with("flags"))
                .any(|line| line.split_whitespace().any(|flag| flag == "ospke"))
        })
        .unwrap_or(false)
}

#[test]
fn an_execute_only_page_is_readable_with_the_library() {
    let lib = library();
    assert!(lib.is_file(), "{} was not built", lib.display());
    let status = probe().env("LD_PRELOAD", &lib).status().unwrap();
    assert!(status.success(), "probe with the library: {status}");
}

#[test]
fn without_the_library_the_read_faults_on_pku_cpus() {
    if !has_pku() {
        eprintln!("skipped: this CPU has no memory protection keys");
        return;
    }
    let status = probe().status().unwrap();
    assert!(
        !status.success(),
        "the read should fault without the library"
    );
}
