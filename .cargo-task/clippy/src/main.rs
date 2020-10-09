/*
@ct-help@ Run "cargo clippy" to check for lint. @@

@ct-task-deps@
setup
@@
*/

mod cargo_task_util;

fn main() {
    let env = cargo_task_util::ct_env();
    let mut cmd = env.cargo();
    cmd.arg("clippy");
    env_check_fatal!(env, env.exec(cmd));
}
