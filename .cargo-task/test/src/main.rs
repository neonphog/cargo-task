//! Make sure we can also still run full-directory-style tasks.

/*
@ct-help@ Run "cargo test". @@
*/

use cargo_task_util::*;

fn main() {
    let env = ct_env();
    let mut cmd = env.cargo();
    cmd.arg("test");
    cmd.arg("--all-features");
    ct_check_fatal!(env.exec(cmd));
}
