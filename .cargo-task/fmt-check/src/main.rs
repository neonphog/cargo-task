/*
@ct-help@ Run "cargo fmt --check" enforce style. @@

@ct-task-deps@
setup
@@
*/

mod cargo_task_util;
use cargo_task_util::*;

fn main() {
    let env = ct_env();
    let mut cmd = env.cargo();
    cmd.arg("fmt");
    cmd.arg("--");
    cmd.arg("--check");
    ct_check_fatal!(env.exec(cmd));
}
