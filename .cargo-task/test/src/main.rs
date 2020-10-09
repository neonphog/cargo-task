/*
@ct-help@ Run "cargo test". @@

@ct-task-deps@
setup
@@
*/

mod cargo_task_util;

fn main() {
    let env = cargo_task_util::ct_env();
    let mut cmd = env.cargo();
    cmd.arg("test");
    cmd.arg("--all-features");
    env_check_fatal!(env, env.exec(cmd));
}
