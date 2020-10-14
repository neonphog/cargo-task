use crate::{cargo_task_util::*, CARGO_TASK_DIR};

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

fn set_env<N: AsRef<OsStr>, V: AsRef<OsStr>>(n: N, v: V) {
    std::env::set_var(n, v);
}

/// Delete any / all 'CT_' environment variables,
/// in preparation for setting new ones.
fn clear() {
    for (k, _v) in std::env::vars_os() {
        let i = k.to_string_lossy();
        let mut i = i.chars();
        if i.next() == Some('C')
            && i.next() == Some('T')
            && i.next() == Some('_')
        {
            std::env::remove_var(k);
        }
    }
}

/// Gather understanding of our cargo-task location.
/// Translate it all into environment variables that CTEnv can read.
pub fn load() -> Result<(), ()> {
    clear();

    // cargo binary path
    let cargo_path = std::env::var_os("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("cargo"));
    set_env("CARGO", cargo_path);

    // work_dir
    let work_dir = find_cargo_task_work_dir()?;
    set_env("CT_WORK_DIR", &work_dir);

    // cargo task path
    let mut cargo_task_path = work_dir;
    cargo_task_path.push(CARGO_TASK_DIR);
    set_env("CT_PATH", &cargo_task_path);

    // cargo task target dir
    let mut cargo_task_target = cargo_task_path.clone();
    cargo_task_target.push("target");
    if let Some(target) = std::env::var_os("CT_TARGET") {
        cargo_task_target = PathBuf::from(target);
    }
    set_env("CT_TARGET", &cargo_task_target);

    // cli arguments
    let mut tasks = Vec::new();
    let mut args = Vec::new();
    let mut found_sep = false;
    for (idx, arg) in std::env::args().enumerate() {
        if idx < 2 {
            continue;
        }
        if !found_sep && arg == "--" {
            found_sep = true;
            continue;
        }
        if found_sep {
            args.push(arg);
        } else {
            tasks.push(arg);
        }
    }
    set_env("CT_TASKS", tasks.join(" "));
    set_env("CT_ARGS", args.join(" "));

    // load cargo-task tasks
    let tasks = enumerate_task_metadata(&cargo_task_path);
    for (_, task) in tasks {
        let path_name = format!("CT_TASK_{}_PATH", task.name);
        set_env(&path_name, &task.path);
        if task.is_script {
            let script_name = format!("CT_TASK_{}_IS_SCRIPT", task.name);
            set_env(&script_name, "1");
        }
        if task.default {
            let def_name = format!("CT_TASK_{}_DEFAULT", task.name);
            set_env(&def_name, "1");
        }
        if task.bootstrap {
            let bs_name = format!("CT_TASK_{}_BOOTSTRAP", task.name);
            set_env(&bs_name, "1");
        }
        if !task.help.is_empty() {
            let def_name = format!("CT_TASK_{}_HELP", task.name);
            set_env(&def_name, &task.help);
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
            set_env(&deps_name, &task_deps);
        }
    }

    Ok(())
}

/// Searches up the directories from the current dir,
/// looking for a directory containing a '.cargo-task' directory.
fn find_cargo_task_work_dir() -> Result<PathBuf, ()> {
    let mut cargo_task_path = std::fs::canonicalize(".").map_err(|_| ())?;

    loop {
        for item in std::fs::read_dir(&cargo_task_path).map_err(|_| ())? {
            if let Ok(item) = item {
                if !item.file_type().map_err(|_| ())?.is_dir() {
                    continue;
                }
                if item.file_name() == CARGO_TASK_DIR {
                    return Ok(cargo_task_path);
                }
            }
        }

        if !cargo_task_path.pop() {
            break;
        }
    }

    Err(())
}

/// Searches CARGO_TASK_DIR for defined tasks, and loads up metadata.
fn enumerate_task_metadata<P: AsRef<Path>>(
    cargo_task_path: P,
) -> BTreeMap<String, CTTaskMeta> {
    let mut out = BTreeMap::new();

    for item in
        std::fs::read_dir(&cargo_task_path).expect("failed to read directory")
    {
        if let Ok(item) = item {
            if !item.file_type().expect("failed to read file type").is_dir()
                || item.file_name() == "target"
            {
                continue;
            }
            let path = item.path();
            let mut main_path = path.clone();
            main_path.push("src");
            main_path.push("main.rs");
            let meta = match parse_metadata(&main_path) {
                Ok(meta) => meta,
                Err(_) => {
                    crate::ct_warn!(
                        "could not parse task {:?}",
                        item.file_name()
                    );
                    continue;
                }
            };
            let meta = CTTaskMeta {
                name: item
                    .file_name()
                    .into_string()
                    .expect("failed to convert filename to string"),
                is_script: false,
                path,
                default: meta.default,
                bootstrap: meta.bootstrap,
                help: meta.help,
                task_deps: meta.task_deps,
            };
            out.insert(meta.name.clone(), meta);
        }
    }

    out
}

struct Meta {
    default: bool,
    bootstrap: bool,
    task_deps: Vec<String>,
    help: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            default: false,
            bootstrap: false,
            task_deps: Vec::new(),
            help: "".to_string(),
        }
    }
}

/// Parse meta-data info from the rust main source file.
fn parse_metadata<P: AsRef<Path>>(path: P) -> Result<Meta, ()> {
    let mut meta = Meta::default();

    let file = std::fs::File::open(&path).map_err(|_| ())?;
    let mut parser = crate::at_at::AtAtParser::new(file);
    while let Some(items) = parser.parse() {
        for item in items {
            if let crate::at_at::AtAtParseItem::KeyValue(k, v) = item {
                match k.as_str() {
                    "ct-default" => {
                        if v == "true" {
                            meta.default = true;
                        }
                    }
                    "ct-bootstrap" => {
                        if v == "true" {
                            meta.bootstrap = true;
                        }
                    }
                    "ct-dependencies" => {
                        crate::ct_fatal!("deps not allowed in full directory tasks - just specify them in your Cargo.toml");
                    }
                    "ct-task-deps" => {
                        for dep in v.split_whitespace() {
                            meta.task_deps.push(dep.to_string());
                        }
                    }
                    "ct-help" => {
                        meta.help = v;
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(meta)
}
