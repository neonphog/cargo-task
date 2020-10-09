use crate::cargo_task_util::*;

use crate::cargo_task_util::CTEnv;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    rc::Rc,
};

/// The .cargo-task directory name
const CARGO_TASK_DIR: &str = ".cargo-task";

fn set_env<N: AsRef<OsStr>, V: AsRef<OsStr>>(n: N, v: V) {
    std::env::set_var(n, v);
}

/// Gather understanding of our cargo-task location.
/// Translate it all into environment variables that CTEnv can read.
pub fn load(fake_env: Rc<CTEnv>) -> Result<(), ()> {
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

    // cli arguments
    let mut args = std::env::args().collect::<Vec<_>>();
    args.drain(..std::cmp::min(args.len(), 2));
    if !args.is_empty() {
        set_env("CT_CUR_ARGS", args.join(" "));
    }

    // load cargo-task tasks
    let tasks = enumerate_task_metadata(fake_env, &cargo_task_path);
    for (_, task) in tasks {
        let path_name = format!("CT_TASK_{}_PATH", task.name);
        set_env(&path_name, &task.path);
        if task.default {
            let def_name = format!("CT_TASK_{}_DEFAULT", task.name);
            set_env(&def_name, "1");
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
    fake_env: Rc<CTEnv>,
    cargo_task_path: P,
) -> BTreeMap<String, CTTaskMeta> {
    let mut out = BTreeMap::new();

    for item in
        std::fs::read_dir(&cargo_task_path).expect("failed to read directory")
    {
        if let Ok(item) = item {
            if !item.file_type().expect("failed to read file type").is_dir() {
                continue;
            }
            let path = item.path();
            let mut main_path = path.clone();
            main_path.push("src");
            main_path.push("main.rs");
            let meta = parse_metadata(fake_env.clone(), &main_path);
            let meta = CTTaskMeta {
                name: item
                    .file_name()
                    .into_string()
                    .expect("failed to convert filename to string"),
                path,
                default: meta.default,
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
    task_deps: Vec<String>,
    help: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            default: false,
            task_deps: Vec::new(),
            help: "".to_string(),
        }
    }
}

/// Parse meta-data info from the rust main source file.
fn parse_metadata<P: AsRef<Path>>(fake_env: Rc<CTEnv>, path: P) -> Meta {
    let mut meta = Meta::default();

    let file = crate::env_check_fatal!(&fake_env, std::fs::File::open(&path));
    let mut parser = crate::at_at::AtAtParser::new(fake_env.clone(), file);
    while let Some(items) = parser.parse() {
        for item in items {
            if let crate::at_at::AtAtParseItem::KeyValue(k, v) = item {
                match k.as_str() {
                    "ct-default" => {
                        if v == "true" {
                            meta.default = true;
                        }
                    }
                    "ct-dependencies" => {
                        crate::env_fatal!(&fake_env, "deps not allowed in full directory tasks - just specify them in your Cargo.toml");
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

    meta
}
