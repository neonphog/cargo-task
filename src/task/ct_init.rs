use crate::*;

/// This task runs before environment loading.
/// If we don't already have a .cargo-task directory - create one.
pub fn ct_init() {
    ct_info!("Initializing current directory for cargo-task...");
    if let Ok(meta) = std::fs::metadata(CARGO_TASK_DIR) {
        if meta.is_dir() {
            ct_fatal!("'{}' already exists, aborting.", CARGO_TASK_DIR);
        }
    }
    let _ = std::fs::create_dir(CARGO_TASK_DIR);
    ct_check_fatal!(std::fs::write(CT_DIR_GIT_IGNORE, CT_DIR_GIT_IGNORE_SRC));
}
