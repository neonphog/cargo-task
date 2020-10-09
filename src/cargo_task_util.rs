//! Common cargo_task mod will be available to all tasks.

use std::{collections::BTreeMap, ffi::OsString, path::PathBuf, rc::Rc};

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

    /// Use colored log output.
    pub with_color: bool,

    /// Current args to cargo-task.
    pub cur_args: Vec<String>,

    /// All tasks defined in the task directory.
    pub tasks: BTreeMap<String, CTTaskMeta>,
}

impl CTEnv {
    /// Create a new cargo std::process::Command
    pub fn cargo(&self) -> std::process::Command {
        std::process::Command::new(&self.cargo_path)
    }

    /// Execute a rust std::process::Command
    pub fn exec(&self, mut cmd: std::process::Command) -> std::io::Result<()> {
        let non_zero_err = format!("{:?} exited non-zero", cmd);
        cmd.stdin(std::process::Stdio::inherit());
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
        if !cmd.spawn()?.wait()?.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                non_zero_err,
            ));
        }
        Ok(())
    }

    /// Log to stdout at INFO log level.
    pub fn info(&self, text: &str) {
        self.priv_log(LogLevel::Info, text);
    }

    /// Log to stderr at WARN log level.
    pub fn warn(&self, text: &str) {
        self.priv_log(LogLevel::Warn, text);
    }

    /// Log to stderr at FATAL log level and exit the process with error code.
    pub fn fatal(&self, text: &str) -> ! {
        self.priv_log(LogLevel::Fatal, text);
        std::process::exit(1)
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

    /// help info for this task
    pub help: String,

    /// any cargo-task task dependencies
    pub task_deps: Vec<String>,
}

// -- private -- //

thread_local! {
    static CT_ENV: Rc<CTEnv> = priv_new_env();
}

fn priv_new_env() -> Rc<CTEnv> {
    let with_color = std::env::var_os("CT_WITH_COLOR").is_some();
    let fake_env_for_logging = CTEnv {
        cargo_path: PathBuf::new(),
        work_dir: PathBuf::new(),
        cargo_task_path: PathBuf::new(),
        with_color,
        cur_args: Vec::with_capacity(0),
        tasks: BTreeMap::new(),
    };

    let cargo_path = match std::env::var_os("CARGO").map(PathBuf::from) {
        Some(cargo_path) => cargo_path,
        None => fake_env_for_logging
            .fatal("CARGO binary path not set in environment"),
    };
    let work_dir = match std::env::var_os("CT_WORK_DIR").map(PathBuf::from) {
        Some(work_dir) => work_dir,
        None => fake_env_for_logging
            .fatal("CT_WORK_DIR environment variable not set"),
    };
    let cargo_task_path = match std::env::var_os("CT_PATH").map(PathBuf::from) {
        Some(cargo_task_path) => cargo_task_path,
        None => {
            fake_env_for_logging.fatal("CT_PATH environment variable not set")
        }
    };
    let cur_args = match std::env::var_os("CT_CUR_ARGS") {
        Some(args) => args
            .to_string_lossy()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        None => Vec::with_capacity(0),
    };
    let tasks = match enumerate_task_metadata() {
        Ok(tasks) => tasks,
        Err(e) => fake_env_for_logging.fatal(e),
    };

    Rc::new(CTEnv {
        cargo_path,
        work_dir,
        cargo_task_path,
        with_color,
        cur_args,
        tasks,
    })
}

impl CTEnv {
    fn priv_log(&self, level: LogLevel, text: &str) {
        for line in text.split('\n') {
            self.priv_log_line(level, line);
        }
    }

    fn priv_log_line(&self, level: LogLevel, text: &str) {
        let task_name = std::env::var_os("CT_CUR_TASK")
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "".to_string());
        let t_colon = if task_name.is_empty() { "" } else { ":" };
        let base = if self.with_color { "\x1b[97m" } else { "" };
        let reset = if self.with_color { "\x1b[0m" } else { "" };
        let (lvl, log) = match level {
            LogLevel::Info => ("INFO", "\x1b[92m"),
            LogLevel::Warn => ("WARN", "\x1b[93m"),
            LogLevel::Fatal => ("FATAL", "\x1b[91m"),
        };
        let log = if self.with_color { log } else { "" };
        if let LogLevel::Info = level {
            println!(
                "{}[ct:{}{}{}{}{}]{} {}",
                base, log, lvl, base, t_colon, task_name, reset, text
            );
        } else {
            eprintln!(
                "{}[ct:{}{}{}{}{}]{} {}",
                base, log, lvl, base, t_colon, task_name, reset, text
            );
        }
    }
}

/// format! style helper for printing out info messages.
#[macro_export]
macro_rules! env_info {
    ($env:expr, $($tt:tt)*) => { $env.info(&format!($($tt)*)); };
}

/// format! style helper for printing out warn messages.
#[macro_export]
macro_rules! env_warn {
    ($env:expr, $($tt:tt)*) => { $env.warn(&format!($($tt)*)); };
}

/// format! style helper for printing out fatal messages.
#[macro_export]
macro_rules! env_fatal {
    ($env:expr, $($tt:tt)*) => { $env.fatal(&format!($($tt)*)); };
}

/// takes an env and a result, if the result is error, runs env_fatal!
#[macro_export]
macro_rules! env_check_fatal {
    ($env:expr, $code:expr) => {
        match { $code } {
            Err(e) => $crate::env_fatal!($env, "{:?}", e),
            Ok(r) => r,
        }
    };
}

/// Loads task metadata from environment.
fn enumerate_task_metadata(
) -> Result<BTreeMap<String, CTTaskMeta>, &'static str> {
    let mut out = BTreeMap::new();

    let env = std::env::vars_os().collect::<BTreeMap<OsString, OsString>>();
    for (env_k, env_v) in env.iter() {
        let env_k = env_k.to_string_lossy();
        if env_k.starts_with("CT_TASK_") && env_k.ends_with("_PATH") {
            let name = env_k[8..env_k.len() - 5].to_string();
            let def_name = format!("CT_TASK_{}_DEFAULT", name);
            let default = env.contains_key(&OsString::from(def_name));
            let help_name = format!("CT_TASK_{}_HELP", name);
            let help = env
                .get(&OsString::from(help_name))
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "".to_string());
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
                    help,
                    task_deps,
                },
            );
        }
    }

    Ok(out)
}

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
    Fatal,
}
