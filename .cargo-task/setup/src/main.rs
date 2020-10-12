/*
@ct-help@ Rebuild tasks if cargo_task_util.rs is updated. @@
*/

use std::path::Path;

mod cargo_task_util;

fn mtime<P: AsRef<Path>>(p: P) -> Result<std::time::SystemTime, ()> {
    Ok(std::fs::metadata(p.as_ref())
        .map_err(|_| ())?
        .modified()
        .map_err(|_| ())?)
}

fn main() {
    let root_time = std::fs::metadata("src/cargo_task_util.rs")
        .unwrap()
        .modified()
        .unwrap();

    // In this particular crate - when testing ourselves -
    // the task binaries can get cached with an old cargo_task_util.rs
    // so if that file has been updated we need to touch the task file.
    for task in std::fs::read_dir(".cargo-task").unwrap() {
        if let Ok(task) = task {
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
                    let mut f = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(&c_path)
                        .unwrap();
                    // just opening it doesn't seem to update the mod time
                    // append a newline, then remove it.
                    std::io::Write::write_all(&mut f, b"\n").unwrap();
                    f.sync_all().unwrap();
                    let len = f.metadata().unwrap().len();
                    f.set_len(len - 1).unwrap();
                    f.sync_all().unwrap();
                    drop(f);
                }
            }
        }
    }
}
