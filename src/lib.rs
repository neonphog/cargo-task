#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.
//!
//! ## Usage
//!
//! - Install the cargo subcommand:
//! ```shell
//! cargo install cargo-task
//! ```
//!
//! - Initialize your rust repository with a .cargo-task directory:
//! ```shell
//! cargo task ct-init
//! ```
//!
//! - Change to that directory:
//! ```shell
//! cd .cargo-task
//! ```
//!
//! - Create a new task project:
//! ```shell
//! cargo new --bin my-task
//! ```
//!
//! - Edit `.cargo-task/my-task/src/main.rs` to look like:
//! ```shell
//! /*
//! @ct-default@ true @@
//! */
//!
//! mod cargo_task_util;
//!
//! fn main() {
//!     ct_info!("Hello World!");
//! }
//! ```
//!
//! - Return to your root directory:
//! ```shell
//! cd ..
//! ```
//!
//! - Run 'cargo task
//! ```shell
//! cargo task
//! ```

pub mod at_at;
pub mod cargo_task_util;
mod env_loader;

/// The .cargo-task directory name
const CARGO_TASK_DIR: &str = ".cargo-task";

/// The .cargo-task/.gitignore file
const CT_DIR_GIT_IGNORE: &str = ".cargo-task/.gitignore";

/// Source-code content of cargo_task_util.rs
const CARGO_TASK_UTIL_SRC: &[u8] = include_bytes!("cargo_task_util.rs");

/// Source content of .cargo-task/.gitignore for 'ct-init'
const CT_DIR_GIT_IGNORE_SRC: &[u8] =
    include_bytes!("../.cargo-task/.gitignore");

mod exec;
pub use exec::*;
