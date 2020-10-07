#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! Cargo Task Library

pub mod cargo_task_util;
mod env_loader;

use std::path::{Path, PathBuf};

const CARGO_TASK_UTIL_SRC: &[u8] = include_bytes!("cargo_task_util.rs");

/// Main entrypoint for cargo-task binary.
pub fn exec_cargo_task() {
    env_loader::load();
    let env = cargo_task_util::ct_env();
    println!("env: {:#?}", env);
    run_task(&env, "fmt-check");
    run_task(&env, "clippy");
    run_task(&env, "test");
}

/// run a specific task
fn run_task(env: &cargo_task_util::CTEnv, task_name: &str) {
    let task = task_build(&env, task_name);

    println!("# CARGO-TASK - '{}' #", task_name);

    let mut cmd = std::process::Command::new(task);
    cmd.current_dir(&env.work_dir);
    env.exec(cmd);

    println!("# CARGO-TASK - '{}' complete #", task_name);
}

/// build a specific task crate
fn task_build(env: &cargo_task_util::CTEnv, task_name: &str) -> PathBuf {
    let mut task_dir = env.cargo_task_path.clone();
    task_dir.push(task_name);

    let mut target_dir = env.work_dir.clone();
    target_dir.push("target");

    let mut artefact_path = target_dir.clone();
    artefact_path.push("release");
    artefact_path.push(task_name);

    if let Ok(meta) = std::fs::metadata(&artefact_path) {
        let artefact_time = meta
            .modified()
            .expect("failed to get artifact modified time");
        let dir_time = get_newest_time(&task_dir);

        if artefact_time >= dir_time {
            return artefact_path;
        }
    }

    println!("# CARGO-TASK - '{}' building #", task_name);

    let mut workspace = env.cargo_task_path.clone();
    workspace.push("Cargo.toml");
    let _ = std::fs::remove_file(&workspace);
    std::fs::write(
        &workspace,
        format!(
            r#"[workspace]
members = [
    "{}",
]"#,
            task_name
        ),
    )
    .expect("failed to write workspace Cargo.toml");

    let mut util = task_dir.clone();
    util.push("src");
    util.push("cargo_task_util.rs");
    let _ = std::fs::remove_file(&util);
    std::fs::write(&util, CARGO_TASK_UTIL_SRC).expect("failed to write cargo_task_util.rs");

    let mut cmd = env.cargo();
    cmd.arg("build");
    cmd.arg("--release");

    let mut manifest_path = task_dir;
    manifest_path.push("Cargo.toml");

    cmd.arg("--manifest-path");
    cmd.arg(manifest_path);

    cmd.arg("--target-dir");
    cmd.arg(&target_dir);

    env.exec(cmd);

    let _ = std::fs::remove_file(&workspace);
    let _ = std::fs::remove_file(&util);

    artefact_path
}

/// recursively get the newest update time for any file/dir
fn get_newest_time<P: AsRef<Path>>(path: P) -> std::time::SystemTime {
    let mut newest_time = std::time::SystemTime::UNIX_EPOCH;

    for item in std::fs::read_dir(&path).expect("failed to read directory") {
        if let Ok(item) = item {
            let t = item.file_type().expect("failed to get file type");

            if t.is_dir() {
                let updated = get_newest_time(item.path());
                if updated > newest_time {
                    newest_time = updated;
                }
            } else if t.is_file() {
                let updated = item
                    .metadata()
                    .expect("failed to get metadata")
                    .modified()
                    .expect("failed to get modified time");
                if updated > newest_time {
                    newest_time = updated;
                }
            }
        }
    }

    newest_time
}
