#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
//! cargo-task binary

/// cargo-task entrypoint
pub fn main() {
    cargo_task::exec_cargo_task();
}
