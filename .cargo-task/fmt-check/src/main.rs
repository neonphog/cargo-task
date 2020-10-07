/*
# this is not a default task
@ct-default@ false @@

# this task depends on the serde crate
@ct-dependencies@
serde = { version = "1", features = [ "derive" ] }
@@

# two other cargo-task tasks should be run before this task
@ct-task-deps@
task1
task2
@@
*/

mod cargo_task_util;

fn main() {
    let env = cargo_task_util::ct_env();
    let mut cmd = env.cargo();
    cmd.arg("fmt");
    cmd.arg("--");
    cmd.arg("--check");
    env.exec(cmd);
}
