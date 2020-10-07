use crate::cargo_task_util::*;

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// The .cargo-task directory name
const CARGO_TASK_DIR: &str = ".cargo-task";

fn set_env_if_none<N: AsRef<OsStr>, V: AsRef<OsStr>>(n: N, v: V) {
    if std::env::var_os(n.as_ref()).is_none() {
        std::env::set_var(n, v);
    }
}

/// Gather understanding of our cargo-task location.
/// Translate it all into environment variables that CTEnv can read.
pub fn load() {
    // cargo binary path
    let cargo_path = std::env::var_os("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cargo"));
    set_env_if_none("CARGO", cargo_path);

    // cargo task path
    let work_dir = find_cargo_task_work_dir();
    set_env_if_none("CT_WORK_DIR", &work_dir);

    let mut cargo_task_path = work_dir;
    cargo_task_path.push(CARGO_TASK_DIR);
    set_env_if_none("CT_PATH", &cargo_task_path);

    // load cargo-task tasks
    let tasks = enumerate_task_metadata(&cargo_task_path);
    for (_, task) in tasks {
        let path_name = format!("CT_TASK_{}_PATH", task.name);
        set_env_if_none(&path_name, &task.path);
        if task.default {
            let def_name = format!("CT_TASK_{}_DEFAULT", task.name);
            set_env_if_none(&def_name, "1");
        }
        let mut task_deps = "".to_string();
        for task_dep in task.task_deps.iter() {
            if !task_deps.is_empty() {
                task_deps.push_str(" ");
            }
            task_deps.push_str(task_dep);
        }
        if !task_deps.is_empty() {
            let deps_name = format!("CT_TASK_{}_TASK_DEPS", task.name);
            set_env_if_none(&deps_name, &task_deps);
        }
    }
}

/// Searches up the directories from the current dir,
/// looking for a directory containing a '.cargo-task' directory.
fn find_cargo_task_work_dir() -> PathBuf {
    let mut cargo_task_path =
        std::fs::canonicalize(".").expect("faile to canonicalize current directory");

    loop {
        for item in std::fs::read_dir(&cargo_task_path).expect("failed to read directory") {
            if let Ok(item) = item {
                if !item.file_type().expect("failed to read file type").is_dir() {
                    continue;
                }
                if item.file_name() == CARGO_TASK_DIR {
                    return cargo_task_path;
                }
            }
        }

        if !cargo_task_path.pop() {
            break;
        }
    }

    eprintln!("ERROR: Could not find '{}' directory.", CARGO_TASK_DIR);
    eprintln!("Have you run 'cargo task ct-init'?");
    std::process::exit(1);
}

/// Searches CARGO_TASK_DIR for defined tasks, and loads up metadata.
fn enumerate_task_metadata<P: AsRef<Path>>(cargo_task_path: P) -> BTreeMap<String, CTTaskMeta> {
    let mut out = BTreeMap::new();

    for item in std::fs::read_dir(&cargo_task_path).expect("failed to read directory") {
        if let Ok(item) = item {
            if !item.file_type().expect("failed to read file type").is_dir() {
                continue;
            }
            let path = item.path();
            let mut main_path = path.clone();
            main_path.push("src");
            main_path.push("main.rs");
            let meta = parse_metadata(&main_path);
            let meta = CTTaskMeta {
                name: item
                    .file_name()
                    .into_string()
                    .expect("failed to convert filename to string"),
                path,
                default: meta.default,
                task_deps: meta.task_deps,
            };
            out.insert(meta.name.clone(), meta);
        }
    }

    out
}

struct Meta {
    default: bool,
    task_deps: Vec<String>,
}

/// Parse meta-data info from the rust main source file.
fn parse_metadata<P: AsRef<Path>>(path: P) -> Meta {
    let data = std::fs::read(path).expect("failed to read main.rs");

    // super naive parsing : )
    enum State {
        Waiting,
        LineStart,
        GatherName(Vec<u8>),
        GatherValue(Vec<u8>, Vec<u8>),
        FirstAt(Vec<u8>, Vec<u8>),
    };

    let mut state = State::LineStart;

    let mut nv = Vec::new();

    for c in data {
        state = match state {
            State::Waiting => {
                if c == 10 || c == 13 {
                    State::LineStart
                } else {
                    State::Waiting
                }
            }
            State::LineStart => {
                if c == 64 {
                    State::GatherName(Vec::new())
                } else {
                    State::Waiting
                }
            }
            State::GatherName(mut name) => {
                if c == 64 {
                    State::GatherValue(name, Vec::new())
                } else {
                    name.push(c);
                    State::GatherName(name)
                }
            }
            State::GatherValue(name, mut value) => {
                if c == 64 {
                    State::FirstAt(name, value)
                } else {
                    value.push(c);
                    State::GatherValue(name, value)
                }
            }
            State::FirstAt(name, mut value) => {
                if c == 64 {
                    nv.push((
                        String::from_utf8_lossy(&name).trim().to_string(),
                        String::from_utf8_lossy(&value).trim().to_string(),
                    ));
                    State::Waiting
                } else {
                    value.push(64);
                    State::GatherValue(name, value)
                }
            }
        }
    }

    let mut default = false;
    let mut task_deps = Vec::new();

    for (n, v) in nv {
        match n.as_str() {
            "ct-default" => {
                if v == "true" {
                    default = true;
                }
            }
            "ct-dependencies" => {
                println!("ignoring cargo deps for now:\n{}\n", v);
            }
            "ct-task-deps" => {
                for dep in v.split_whitespace() {
                    task_deps.push(dep.to_string());
                }
            }
            _ => (),
        }
    }

    Meta { default, task_deps }
}
