/*
# this IS a default task
@ct-default@ true @@

# some help info for 'cargo help task'
@ct-help@ This meta default task just ensures other tasks run. @@


@ct-task-deps@
setup
fmt-check
clippy
test
readme
@@
*/

mod cargo_task_util;
//use cargo_task_util::*;

fn main() {
    ct_info!("default task is a no-op");
}
