/*
# two other cargo-task tasks should be run before this task
@ct-task-deps@
setup
@@
*/

mod cargo_task_util;

fn main() {
    let env = cargo_task_util::ct_env();
    let mut cmd = env.cargo();
    cmd.arg("fmt");
    cmd.arg("--");
    cmd.arg("--check");
    env_check_fatal!(env, env.exec(cmd));
}
