#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! cargo-task binary

/// cargo-task entrypoint
pub fn main() {
    cargo_task::exec_cargo_task();
}
