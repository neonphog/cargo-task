use crate::{_cargo_task_util::*, CARGO_TASK_DIR, *};

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
pub fn load() -> Result<(), &'static str> {
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
        if let Some(min_version) = &task.min_version {
            let mv_name = format!("CT_TASK_{}_MIN_VER", task.name);
            set_env(&mv_name, min_version);
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
        if let Some(cargo_deps) = task.cargo_deps {
            let deps_name = format!("CT_TASK_{}_CARGO_DEPS", task.name);
            set_env(&deps_name, cargo_deps);
        }
        let mut task_deps = "".to_string();
        for task_dep in task.task_deps.iter() {
            if !task_deps.is_empty() {
                task_deps.push(' ');
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
fn find_cargo_task_work_dir() -> Result<PathBuf, &'static str> {
    const E: &str = "failed to find .cargo-task dir";
    let mut cargo_task_path = std::env::current_dir().map_err(|_| E)?;

    loop {
        for item in std::fs::read_dir(&cargo_task_path)
            .map_err(|_| E)?
            .flatten()
        {
            if !item.file_type().map_err(|_| E)?.is_dir() {
                continue;
            }
            if item.file_name() == CARGO_TASK_DIR {
                return Ok(cargo_task_path);
            }
        }

        if !cargo_task_path.pop() {
            break;
        }
    }

    Err(E)
}

/// Searches CARGO_TASK_DIR for defined tasks, and loads up metadata.
fn enumerate_task_metadata<P: AsRef<Path>>(
    cargo_task_path: P,
) -> BTreeMap<String, CTTaskMeta> {
    let mut out = BTreeMap::new();

    for item in std::fs::read_dir(&cargo_task_path)
        .expect("failed to read directory")
        .flatten()
    {
        let file_name = item.file_name().to_string_lossy().to_string();
        if &file_name == "cargo_task_util"
            || &file_name == "target"
            || file_name.starts_with('.')
        {
            continue;
        }

        let file_type = ct_check_fatal!(item.file_type());

        if file_type.is_file() && file_name.ends_with(".ct.rs") {
            let path = item.path();
            let meta = ct_check_fatal!(parse_metadata(&path));
            let meta = CTTaskMeta {
                name: file_name[..file_name.len() - 6].to_string(),
                is_script: true,
                min_version: meta.min_version,
                path,
                default: meta.default,
                bootstrap: meta.bootstrap,
                help: meta.help,
                cargo_deps: meta.cargo_deps,
                task_deps: meta.task_deps,
            };
            out.insert(meta.name.clone(), meta);
        } else if file_type.is_dir() {
            let path = item.path();
            let mut main_path = path.clone();
            main_path.push("src");
            main_path.push("main.rs");
            let meta = ct_check_fatal!(parse_metadata(&main_path));
            if meta.cargo_deps.is_some() {
                ct_fatal!("@ct-cargo-deps@ are illegal in directory-style task crates - just specify your deps in your Cargo.toml file");
            }
            let meta = CTTaskMeta {
                name: file_name,
                is_script: false,
                min_version: meta.min_version,
                path,
                default: meta.default,
                bootstrap: meta.bootstrap,
                help: meta.help,
                cargo_deps: None,
                task_deps: meta.task_deps,
            };
            out.insert(meta.name.clone(), meta);
        }
    }

    out
}

struct Meta {
    min_version: Option<String>,
    default: bool,
    bootstrap: bool,
    cargo_deps: Option<String>,
    task_deps: Vec<String>,
    help: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            min_version: None,
            default: false,
            bootstrap: false,
            cargo_deps: None,
            task_deps: Vec::new(),
            help: "".to_string(),
        }
    }
}

/// Parse meta-data info from the rust main source file.
fn parse_metadata<P: AsRef<Path>>(path: P) -> Result<Meta, String> {
    let mut meta = Meta::default();

    let file = std::fs::File::open(&path).map_err(|e| {
        format!("parse metadata error: {:?}: {:?}", path.as_ref(), e,)
    })?;
    let mut parser = at_at::AtAtParser::new(file);
    while let Some(items) = parser.parse() {
        for item in items {
            if let at_at::AtAtParseItem::KeyValue(k, v) = item {
                match k.as_str() {
                    "ct-min-version" => {
                        meta.min_version = Some(v);
                    }
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
                    "ct-cargo-deps" => {
                        meta.cargo_deps = Some(v);
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
