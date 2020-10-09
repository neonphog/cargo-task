/*
@ct-help@ Run "cargo clippy" to check for lint. @@

@ct-task-deps@
setup
@@
*/

mod cargo_task_util;
use cargo_task_util::*;

fn main() {
    let env = ct_env();
    let mut cmd = env.cargo();
    cmd.arg("clippy");
    ct_check_fatal!(env.exec(cmd));
}
