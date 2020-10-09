#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! Cargo Task Library

pub mod at_at;
pub mod cargo_task_util;
mod env_loader;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

const CARGO_TASK_UTIL_SRC: &[u8] = include_bytes!("cargo_task_util.rs");
const CT_DIR_GIT_IGNORE_SRC: &[u8] =
    include_bytes!("../.cargo-task/.gitignore");

/// Main entrypoint for cargo-task binary.
pub fn exec_cargo_task() {
    if env_loader::load().is_err() {
        let args = std::env::args().collect::<Vec<_>>();

        // hack check for --help
        if args.len() >= 3 && &args[2] == "--help" {
            print_usage(None);
            std::process::exit(0);
        }

        // hack check for ct-init
        if args.len() >= 3 && &args[2] == "ct-init" {
            ct_info!("Initializing current directory for cargo-task...");
            let _ = std::fs::create_dir(".cargo-task");
            ct_check_fatal!(std::fs::write(
                ".cargo-task/.gitignore",
                CT_DIR_GIT_IGNORE_SRC
            ));
            std::process::exit(0);
        }

        ct_fatal!(
            r"ERROR: Could not find '.cargo-task' directory.
                  Have you run 'cargo task ct-init'?"
        );
    }
    let env = cargo_task_util::ct_env();
    ct_info!("cargo-task running...");

    let mut task_list = Vec::new();
    for task in env.cur_args.iter() {
        if task == "--help" {
            print_usage(Some(&env));
            std::process::exit(0);
        }
        fill_task_deps(&env, &mut task_list, task.to_string(), HashSet::new());
    }
    if task_list.is_empty() {
        for (task, task_meta) in env.tasks.iter() {
            if task_meta.default {
                fill_task_deps(
                    &env,
                    &mut task_list,
                    task.to_string(),
                    HashSet::new(),
                );
            }
        }
    }

    ct_info!("task order: {:?}", task_list);

    for task in task_list {
        match task.as_str() {
            "ct-init" => {
                ct_fatal!("cargo task already initialized, aborting");
            }
            "ct-meta" => {
                ct_info!("print full cargo-task metadata");
                println!("{:#?}", env);
            }
            _ => run_task(&env, &task),
        }
    }

    ct_info!("cargo-task complete : )");
}

/// fill task deps
fn fill_task_deps(
    env: &cargo_task_util::CTEnv,
    task_list: &mut Vec<String>,
    task: String,
    mut visited: HashSet<String>,
) {
    visited.insert(task.clone());
    if !env.tasks.contains_key(&task) {
        // this may be a psuedo task - add it, but don't check deps
        if !task_list.contains(&task) {
            task_list.push(task);
        }
        return;
    }
    for dep in env.tasks.get(&task).unwrap().task_deps.iter() {
        if visited.contains(dep) {
            ct_fatal!("circular task dependency within {:?}", visited);
        }
        fill_task_deps(env, task_list, dep.to_string(), visited.clone());
    }
    if !task_list.contains(&task) {
        task_list.push(task);
    }
}

/// run a specific task
fn run_task(env: &cargo_task_util::CTEnv, task_name: &str) {
    if !env.tasks.contains_key(task_name) {
        ct_fatal!("invalid task name '{}'", task_name);
    }

    let task = task_build(&env, task_name);

    ct_info!("run task: '{}'", task_name);
    std::env::set_var("CT_CUR_TASK", task_name);

    let mut cmd = std::process::Command::new(task);
    cmd.current_dir(&env.work_dir);
    if let Err(e) = env.exec(cmd) {
        ct_fatal!("{:?}", e);
    }

    std::env::remove_var("CT_CUR_TASK");
}

/// build a specific task crate
fn task_build(env: &cargo_task_util::CTEnv, task_name: &str) -> PathBuf {
    let mut task_dir = env.cargo_task_path.clone();
    task_dir.push(task_name);

    let mut target_dir = env.work_dir.clone();
    target_dir.push("target");

    let mut artifact_path = target_dir.clone();
    artifact_path.push("release");
    artifact_path.push(task_name);

    if let Ok(meta) = std::fs::metadata(&artifact_path) {
        let artifact_time = meta
            .modified()
            .expect("failed to get artifact modified time");
        let dir_time = get_newest_time(&task_dir);

        if artifact_time >= dir_time {
            return artifact_path;
        }
    }

    ct_info!("build task '{}'", task_name);

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
    let mut content = b"#![allow(dead_code)]\n".to_vec();
    content.extend_from_slice(CARGO_TASK_UTIL_SRC);
    std::fs::write(&util, &content)
        .expect("failed to write cargo_task_util.rs");

    let mut cmd = env.cargo();
    cmd.arg("build");
    cmd.arg("--release");

    let mut manifest_path = task_dir;
    manifest_path.push("Cargo.toml");

    cmd.arg("--manifest-path");
    cmd.arg(manifest_path);

    cmd.arg("--target-dir");
    cmd.arg(&target_dir);

    ct_check_fatal!(env.exec(cmd));

    let _ = std::fs::remove_file(&workspace);
    let _ = std::fs::remove_file(&util);

    artifact_path
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

/// Print user-friendly usage info.
fn print_usage(env: Option<&cargo_task_util::CTEnv>) {
    println!(
        r#"
# cargo task usage info #

        cargo help task - this help info
             cargo task - execute all configured default cargo tasks
 cargo task [task-list] - execute a specific list of cargo tasks

# system tasks #

                ct-init - generate a .cargo-task directory + .gitignore
                ct-meta - print meta info about the cargo-task configuration
"#
    );

    if let Some(env) = env {
        println!("# locally-defined tasks (* for default) #\n");

        let mut keys = env.tasks.keys().collect::<Vec<_>>();
        keys.sort();

        for task_name in keys {
            let task = env.tasks.get(task_name.as_str()).unwrap();
            let def = if task.default { "*" } else { " " };
            println!("{:>22}{} - {}", task.name, def, task.help);
        }

        println!();
    }
}
