/*
# always run this setup task befor any other tasks
@ct-bootstrap@ true @@

@ct-help@ Rebuild tasks if cargo_task_util.rs is updated. @@
*/

use std::path::Path;

mod cargo_task_util;
use cargo_task_util::*;

fn mtime<P: AsRef<Path>>(p: P) -> Result<std::time::SystemTime, ()> {
    Ok(std::fs::metadata(p.as_ref())
        .map_err(|_| ())?
        .modified()
        .map_err(|_| ())?)
}

fn main() {
    let env = ct_env();

    // first, set some job env vars : )
    let cpu_count = format!("{}", num_cpus::get());
    env.set_env("CARGO_BUILD_JOBS", &cpu_count);
    env.set_env("NUM_JOBS", &cpu_count);
    env.set_env("MY_TEST_KEY", "MY_TEST_VAL");

    let root_time = std::fs::metadata("src/cargo_task_util.rs")
        .unwrap()
        .modified()
        .unwrap();

    // In this particular crate - when testing ourselves -
    // the task binaries can get cached with an old cargo_task_util.rs
    // so if that file has been updated we need to touch the task file.
    for task in std::fs::read_dir(".cargo-task").unwrap() {
        if let Ok(task) = task {
            if task.file_name() == "target" {
                continue;
            }
            if task.file_type().unwrap().is_dir() {
                let name = task.file_name();
                let mut a_path = std::path::PathBuf::from("./.cargo-task/target/release");
                a_path.push(&name);
                let a_time = match mtime(&a_path) {
                    Err(_) => std::time::SystemTime::UNIX_EPOCH,
                    Ok(t) => t,
                };
                if a_time < root_time {
                    let mut c_path = task.path();
                    c_path.push("Cargo.toml");
                    ct_info!("touching {:?}", &c_path);
                    let mut u_path = c_path.clone();
                    u_path.pop();
                    u_path.push("Cargo.toml2");
                    // just opening the file does not update the
                    // modified time - using copy/rename for now
                    // if anyone has a better idea, open a PR!
                    ct_check_fatal!(std::fs::copy(&c_path, &u_path));
                    ct_check_fatal!(std::fs::remove_file(&c_path));
                    ct_check_fatal!(std::fs::rename(&u_path, &c_path));
                }
            }
        }
    }
}
