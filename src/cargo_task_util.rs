//! Common cargo_task mod will be available to all tasks.

use std::{collections::BTreeMap, ffi::OsString, path::PathBuf, rc::Rc};

thread_local! {
    static CT_ENV: Rc<CTEnv> = {
        let cargo_path = std::env::var_os("CARGO")
            .map(PathBuf::from)
            .expect("CARGO binary path not set in environment");
        let work_dir = std::env::var_os("CT_WORK_DIR")
            .map(PathBuf::from)
            .expect("CT_WORK_DIR environment variable not set");
        let cargo_task_path = std::env::var_os("CT_PATH")
            .map(PathBuf::from)
            .expect("CT_PATH environment variable not set");
        let tasks = enumerate_task_metadata();
        Rc::new(CTEnv {
            cargo_path,
            work_dir,
            cargo_task_path,
            tasks,
        })
    };
}

/// Fetch the current CTEnv
pub fn ct_env() -> Rc<CTEnv> {
    CT_ENV.with(|env| env.clone())
}

/// Cargo-task environment info struct.
#[derive(Debug)]
pub struct CTEnv {
    /// The path to the cargo binary.
    pub cargo_path: PathBuf,

    /// The .cargo-task directory.
    pub cargo_task_path: PathBuf,

    /// The root of the cargo task execution environment.
    pub work_dir: PathBuf,

    /// All tasks defined in the task directory.
    pub tasks: BTreeMap<String, CTTaskMeta>,
}

impl CTEnv {
    /// Create a new cargo std::process::Command
    pub fn cargo(&self) -> std::process::Command {
        std::process::Command::new(&self.cargo_path)
    }

    /// Execute a rust std::process::Command
    pub fn exec(&self, mut cmd: std::process::Command) {
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
        if !cmd
            .spawn()
            .expect("failed to execute command")
            .wait()
            .expect("failed to execute command")
            .success()
        {
            panic!("command exited non-zero");
        }
    }
}

/// Cargo-task task metadata struct.
#[derive(Debug)]
pub struct CTTaskMeta {
    /// task name
    pub name: String,

    /// task "crate" path
    pub path: PathBuf,

    /// does this path run on default `cargo task` execution?
    pub default: bool,

    /// any cargo-task task dependencies
    pub task_deps: Vec<String>,
}

/// Loads task metadata from environment.
fn enumerate_task_metadata() -> BTreeMap<String, CTTaskMeta> {
    let mut out = BTreeMap::new();

    let env = std::env::vars_os().collect::<BTreeMap<OsString, OsString>>();
    for (env_k, env_v) in env.iter() {
        let env_k = env_k.to_string_lossy();
        if env_k.starts_with("CT_TASK_") && env_k.ends_with("_PATH") {
            let name = env_k[8..env_k.len() - 5].to_string();
            let def_name = format!("CT_TASK_{}_DEFAULT", name);
            let default = env.contains_key(&OsString::from(def_name));
            let deps_name = format!("CT_TASK_{}_TASK_DEPS", name);
            let mut task_deps = Vec::new();
            if let Some(deps) = env.get(&OsString::from(deps_name)) {
                for dep in deps.to_string_lossy().split_whitespace() {
                    task_deps.push(dep.to_string());
                }
            }
            let path = PathBuf::from(env_v);
            out.insert(
                name.clone(),
                CTTaskMeta {
                    name,
                    path,
                    default,
                    task_deps,
                },
            );
        }
    }

    out
}
