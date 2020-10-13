#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.
//!
//! - Platform Agnostic - runs on any platform that cargo runs on.
//! - Zero-Dependency - the task manager itself installs almost instantly.
//! - Rust Task Logic - you can choose to inlude dependencies in your tasks.
//! - Take a look at the [.cargo-task](./.cargo-task) in this repo for examples.
//!
//! ## Quick Start - Installation
//!
//! - Install / Initialize `cargo-task`:
//! ```shell
//! # Install the cargo-task cargo submodule:
//! cargo install cargo-task
//!
//! # Initialize your rust repository with a .cargo-task directory:
//! cargo task ct-init
//!
//! # Change to that directory:
//! cd .cargo-task
//!
//! # Create a new task project:
//! cargo new --bin my-task
//! ```
//!
//! - Edit `.cargo-task/my-task/src/main.rs` to look like:
//! ```ignore
//! /*
//! @ct-default@ true @@
//! */
//!
//! // The content of this module is added by the cargo task builder.
//! // It contains helpers like the ct_* logging macros.
//! mod cargo_task_util;
//!
//! fn main() {
//!     ct_info!("Hello World!");
//! }
//! ```
//!
//! - Test it out:
//! ```shell
//! # Return to your root directory:
//! cd ..
//!
//! # Run 'cargo task':
//! cargo task
//! ```
//!
//! ## Quick Start - Command Line API
//!
//! - `cargo help task` - print out some cli help info.
//! - `cargo task ct-init` - initialize a repository with a .cargo-task dir.
//! - `cargo task ct-meta` - print out meta-information about configured tasks.
//! - `cargo task` - execute any "default" tasks if configured.
//! - `cargo task [task-name]` - execute a specific task (or list of tasks).
//!
//! ## AtAt (@@) cargo-task metadata
//!
//! - The first `@` must be the first character on a line!
//! - Use double `@@` to finish the setting.
//!
//! - `@ct-default@` - set to `true` to make the task a default task.
//! ```shell
//! @ct-default@ true @@
//! ```
//!
//! - `@ct-help@` - specify help text to be displayed next to your task on
//! `cargo help task`
//! ```shell
//! @ct-help@
//! This is a description for a task.
//! One line or two lines is fine.
//! @@
//! ```
//!
//! - `@ct-task-deps@` - whitespace delimited list of tasks that should be
//! run before this one.
//! ```shell
//! @ct-task-deps@ task1 task2 @@
//! ```
//!
//! - `@ct-dependencies@` - reserved for when we implement light-weight
//! single-file tasks.

pub mod at_at;
pub mod cargo_task_util;
mod env_loader;
mod task;

/// The .cargo-task directory name
const CARGO_TASK_DIR: &str = ".cargo-task";

/// The .cargo-task/.gitignore file
#[cfg(windows)]
const CT_DIR_GIT_IGNORE: &str = ".cargo-task\\.gitignore";
#[cfg(not(windows))]
const CT_DIR_GIT_IGNORE: &str = ".cargo-task/.gitignore";

/// Source content of .cargo-task/.gitignore for 'ct-init'
#[cfg(windows)]
const CT_DIR_GIT_IGNORE_SRC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "\\.cargo-task\\.gitignore"
));
#[cfg(not(windows))]
const CT_DIR_GIT_IGNORE_SRC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.cargo-task/.gitignore"
));

/// Source-code content of cargo_task_util.rs
const CARGO_TASK_UTIL_SRC: &[u8] = include_bytes!("cargo_task_util.rs");

mod exec;
pub use exec::*;
