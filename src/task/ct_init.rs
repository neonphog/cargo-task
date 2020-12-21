use crate::*;

/// This idempotent task runs before any valid task is executed or on ct_init.
/// - Ensure the .cargo-task directory exists.
/// - Ensure the .cargo-task/.gitignore file is initialized.
/// - Ensure the .cargo-task/cargo_task_util dep crate exists.
pub fn ct_init() {
    check_cargo_task_dir();
    check_gitignore();
    check_cargo_task_util_crate();
}

fn check_cargo_task_dir() {
    if let Ok(meta) = std::fs::metadata(CARGO_TASK_DIR) {
        if meta.is_dir() {
            return;
        }
    }
    ct_info!("Initializing current directory for cargo-task...");
    ct_check_fatal!(std::fs::create_dir_all(CARGO_TASK_DIR));
}

fn check_gitignore() {
    if let Ok(meta) = std::fs::metadata(CT_DIR_GIT_IGNORE) {
        if meta.is_file() {
            return;
        }
    }
    ct_check_fatal!(std::fs::write(CT_DIR_GIT_IGNORE, CT_DIR_GIT_IGNORE_SRC));
}

fn check_cargo_task_util_crate() {
    // ensure the crate directory
    let mut dir = std::path::PathBuf::new();
    dir.push(CARGO_TASK_DIR);
    dir.push("cargo_task_util");
    let _ = std::fs::create_dir_all(&dir);

    // ensure the Cargo.toml
    check_util_cargo_toml();

    // ensure the src/lib.rs
    check_util_lib_rs();

    // ensure the src/cargo_task_util.rs
    check_util_ctu_rs();
}

fn check_util_cargo_toml() {
    let mut cargo_toml = std::path::PathBuf::new();
    cargo_toml.push(CARGO_TASK_DIR);
    cargo_toml.push("cargo_task_util");
    cargo_toml.push("Cargo.toml");

    if let Ok(meta) = std::fs::metadata(&cargo_toml) {
        if meta.is_file() {
            return;
        }
    }

    ct_check_fatal!(std::fs::write(
        &cargo_toml,
        r#"[package]
name = "cargo_task_util"
version = "0.0.1"
edition = "2018"
"#
    ));
}

fn check_util_lib_rs() {
    let mut lib_rs = std::path::PathBuf::new();
    lib_rs.push(CARGO_TASK_DIR);
    lib_rs.push("cargo_task_util");
    lib_rs.push("src");
    let _ = std::fs::create_dir_all(&lib_rs);

    lib_rs.push("lib.rs");

    if let Ok(meta) = std::fs::metadata(&lib_rs) {
        if meta.is_file() {
            return;
        }
    }

    const CONTENT: &str = r#"#![allow(dead_code)]
pub mod _cargo_task_util;
pub use _cargo_task_util::*;
"#;

    ct_check_fatal!(std::fs::write(&lib_rs, CONTENT));
}

fn check_util_ctu_rs() {
    let mut ctu_rs = std::path::PathBuf::new();
    ctu_rs.push(CARGO_TASK_DIR);
    ctu_rs.push("cargo_task_util");
    ctu_rs.push("src");
    let _ = std::fs::create_dir_all(&ctu_rs);

    ctu_rs.push("_cargo_task_util.rs");

    if let Ok(meta) = std::fs::metadata(&ctu_rs) {
        if meta.is_file() {
            return;
        }
    }

    ct_check_fatal!(std::fs::write(&ctu_rs, CARGO_TASK_UTIL_SRC));
}
