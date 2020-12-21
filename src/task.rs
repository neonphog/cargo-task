use crate::*;

mod help;
pub use help::*;
mod version;
pub use version::*;
mod ct_init;
pub use ct_init::*;
mod ct_meta;
pub use ct_meta::*;
mod ct_clean;
pub use ct_clean::*;

/// check to see if we should execute a pre-env-load task
/// if we should - do it and exit
pub fn check_pre_env_task() {
    let mut args = Vec::new();

    for (idx, arg) in std::env::args().enumerate() {
        if idx < 2 {
            continue;
        }
        if arg == "--" {
            break;
        }
        args.push(arg);
    }

    if args.contains(&"--help".to_string()) {
        help();
        std::process::exit(0);
    }

    if args.contains(&"--version".to_string()) {
        version();
        std::process::exit(0);
    }

    if args.contains(&"ct-init".to_string()) {
        ct_init();
        std::process::exit(0);
    }
}

/// if the task name is a system-defined task - run it and return true
/// if not - return false - exec will attempt to run a user-defined task.
pub fn check_system_task(
    task_name: &str,
    env: &cargo_task_util::CTEnv,
) -> bool {
    match task_name {
        "ct-meta" => {
            ct_meta(env);
            true
        }
        "ct-clean" => {
            ct_clean(env);
            true
        }
        _ => false,
    }
}
