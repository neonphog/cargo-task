use crate::*;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Main entrypoint for cargo-task binary.
pub fn exec_cargo_task() {
    // any pre-env-load tasks to execute?
    task::check_pre_env_task();

    // load our environment into environment variables
    if env_loader::load().is_err() {
        ct_fatal!(
            r"ERROR: Could not find '{}' directory.
Have you run 'cargo task ct-init'?",
            CARGO_TASK_DIR,
        );
    }

    // parse environment vars into env struct
    let mut env = cargo_task_util::ct_env();

    ct_info!("cargo-task running...");

    clean_build_workspace(&env);
    let mut did_build_workspace = false;

    // check for bootstrap tasks
    let mut task_list = Vec::new();
    for (task, task_meta) in env.tasks.iter() {
        if task_meta.bootstrap {
            fill_task_deps(
                &env,
                &mut task_list,
                task.to_string(),
                HashSet::new(),
            );
        }
    }

    // if we are bootstrapping
    if !task_list.is_empty() {
        ct_info!("executing bootstrap list: {:?}", task_list);
        for task in task_list {
            if !task::check_system_task(task.as_str(), &env) {
                run_task(&env, &task, &mut did_build_workspace);
            }
        }
        ct_info!("reloading env post-bootstrap");
        if env_loader::load().is_err() {
            ct_fatal!(
                r"ERROR: Could not find '{}' directory.
    Have you run 'cargo task ct-init'?",
                CARGO_TASK_DIR,
            );
        }
        clean_build_workspace(&env);
        did_build_workspace = false;

        env = cargo_task_util::ct_env();
    }

    // load up specified tasks
    let mut task_list = Vec::new();
    for task in env.task_list.iter() {
        fill_task_deps(&env, &mut task_list, task.to_string(), HashSet::new());
    }

    // if no specified tasks - load default tasks
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
        if !task::check_system_task(task.as_str(), &env) {
            run_task(&env, &task, &mut did_build_workspace);
        }
    }

    clean_build_workspace(&env);

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

/// delete the cargo-task build workspace
fn clean_build_workspace(env: &cargo_task_util::CTEnv) {
    let mut ws = env.cargo_task_target.clone();
    ws.push("ct-workspace");
    let _ = std::fs::remove_dir_all(&ws);
}

/// prep the cargo-task build workspace
fn generate_build_workspace(env: &cargo_task_util::CTEnv) {
    let mut all_tasks = Vec::new();
    let mut ws = env.cargo_task_target.clone();
    ws.push("ct-workspace");
    ct_check_fatal!(std::fs::create_dir_all(&ws));
    for (task, task_meta) in env.tasks.iter() {
        all_tasks.push(task);

        let mut task_dir = ws.clone();
        task_dir.push(task);

        if task_meta.is_script {
            ct_check_fatal!(std::fs::create_dir_all(&task_dir));
            let mut cargo_toml = task_dir.clone();
            cargo_toml.push("Cargo.toml");
            let deps = if let Some(deps) = &task_meta.cargo_deps {
                deps
            } else {
                ""
            };
            ct_check_fatal!(std::fs::write(
                &cargo_toml,
                format!(
                    r#"[package]
name = "{}"
version = "0.0.1"
edition = "2018"

[dependencies]
{}
"#,
                    task, deps,
                )
            ));
            let mut src_dir = task_dir.clone();
            src_dir.push("src");
            ct_check_fatal!(std::fs::create_dir_all(&src_dir));
            let mut main_file = src_dir;
            main_file.push("main.rs");
            ct_check_fatal!(std::fs::copy(&task_meta.path, &main_file));
        } else {
            copy_dir(&task_meta.path, &task_dir);
        }

        let mut util = task_dir;
        util.push("src");
        util.push("cargo_task_util.rs");
        let _ = std::fs::remove_file(&util);
        let mut content = b"#![allow(dead_code)]\n".to_vec();
        content.extend_from_slice(CARGO_TASK_UTIL_SRC);
        ct_check_fatal!(std::fs::write(&util, &content));
    }

    ws.push("Cargo.toml");
    ct_check_fatal!(std::fs::write(
        &ws,
        format!(
            r#"[workspace]
members = {:?}
"#,
            all_tasks
        ),
    ));
}

/// recursively copy a whole directory
fn copy_dir<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) {
    ct_check_fatal!(std::fs::create_dir_all(&dest));
    for item in ct_check_fatal!(std::fs::read_dir(src)) {
        if let Ok(item) = item {
            let meta = ct_check_fatal!(item.metadata());
            let mut dest = dest.as_ref().to_owned();
            dest.push(item.file_name());
            if meta.is_dir() {
                copy_dir(item.path(), &dest);
            } else if meta.is_file() {
                ct_check_fatal!(std::fs::copy(item.path(), &dest));
            }
        }
    }
}

/// run a specific task
fn run_task(
    env: &cargo_task_util::CTEnv,
    task_name: &str,
    did_build_workspace: &mut bool,
) {
    if !env.tasks.contains_key(task_name) {
        ct_fatal!("invalid task name '{}'", task_name);
    }

    let task_meta = env.tasks.get(task_name).unwrap();
    if let Some(min_version) = &task_meta.min_version {
        if parse_semver(crate::CARGO_TASK_VER) < parse_semver(min_version) {
            ct_fatal!(
                "cargo-task {} < required min version {}",
                crate::CARGO_TASK_VER,
                min_version,
            );
        }
    }

    let task = task_build(&env, task_name, did_build_workspace);

    ct_info!("run task: '{}'", task_name);
    std::env::set_var("CT_CUR_TASK", task_name);

    let mut cmd = std::process::Command::new(task);
    cmd.current_dir(&env.work_dir);
    for arg in env.arg_list.iter() {
        cmd.arg(arg);
    }
    cmd.stdin(std::process::Stdio::piped());
    let res: Result<(), String> = (move || {
        let mut child = cmd.spawn().map_err(|e| format!("{:?}", e))?;

        // drop stdin to ensure child exit
        drop(child.stdin.take().unwrap());

        let res: Result<(), String> = (|| {
            let status = child.wait().map_err(|e| format!("{:?}", e))?;

            if !status.success() {
                return Err(format!("{} exited non-zero", task_name));
            }

            Ok(())
        })();

        let mut p = env.cargo_task_target.clone();
        let directive_file_name = format!("task-directive-{}.atat", child.id());
        p.push(directive_file_name);

        if let Ok(file) = std::fs::File::open(&p) {
            let mut parser = at_at::AtAtParser::new(file);
            while let Some(res) = parser.parse() {
                for item in res {
                    if let at_at::AtAtParseItem::KeyValue(k, v) = item {
                        match k.as_str() {
                            "ct-set-env" => {
                                let idx = match v.find('=') {
                                    Some(idx) => idx,
                                    None => ct_fatal!(
                                        "no '=' found in ct-set-env directive"
                                    ),
                                };
                                let n = &v[..idx];
                                let v = &v[idx + 1..];
                                std::env::set_var(n, v);
                                ct_info!("CT-SET-ENV: {}={}", n, v);
                            }
                            _ => ct_fatal!("unrecognized AtAt command '{}'", k),
                        }
                    }
                }
            }
        }

        let _ = std::fs::remove_file(&p);

        res
    })();
    std::env::remove_var("CT_CUR_TASK");

    if let Err(e) = res {
        ct_fatal!("{}", e);
    }
}

/// build a specific task crate
fn task_build(
    env: &cargo_task_util::CTEnv,
    task_name: &str,
    did_build_workspace: &mut bool,
) -> PathBuf {
    let task_meta = env.tasks.get(task_name).unwrap();

    let target_dir = env.cargo_task_target.clone();

    let mut artifact_path = target_dir.clone();
    artifact_path.push("release");
    artifact_path.push(task_name);

    if let Ok(meta) = std::fs::metadata(&artifact_path) {
        let artifact_time = meta
            .modified()
            .expect("failed to get artifact modified time");
        let dir_time = get_newest_time(&task_meta.path);

        if artifact_time >= dir_time {
            return artifact_path;
        }
    }

    ct_info!("build task '{}'", task_name);

    if !*did_build_workspace {
        *did_build_workspace = true;
        generate_build_workspace(env);
    }

    let mut crate_path = env.cargo_task_target.clone();
    crate_path.push("ct-workspace");
    crate_path.push(task_name);

    let mut cmd = env.cargo();
    cmd.arg("build");
    cmd.arg("--release");

    let mut manifest_path = crate_path;
    manifest_path.push("Cargo.toml");

    cmd.arg("--manifest-path");
    cmd.arg(manifest_path);

    cmd.arg("--target-dir");
    cmd.arg(&target_dir);

    ct_check_fatal!(env.exec(cmd));

    artifact_path
}

/// recursively get the newest update time for any file/dir
fn get_newest_time<P: AsRef<Path>>(path: P) -> std::time::SystemTime {
    let mut newest_time = std::time::SystemTime::UNIX_EPOCH;

    if let Ok(metadata) = std::fs::metadata(&path) {
        if metadata.is_file() {
            return metadata.modified().expect("failed to get modified time");
        }
    }

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

/// Parse a semver string into a (usize, usize, usize)
fn parse_semver(s: &str) -> (usize, usize, usize) {
    let r = s.split('.').collect::<Vec<_>>();
    if r.len() != 3 {
        ct_fatal!("invalid semver: {}", s);
    }
    (
        r[0].parse().unwrap(),
        r[1].parse().unwrap(),
        r[2].parse().unwrap(),
    )
}
