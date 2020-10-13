/*
# run this if no other tasks are specified
@ct-default@ true @@

# some help info for 'cargo help task'
@ct-help@ This meta default task just ensures other tasks run. @@

@ct-task-deps@
fmt-check
clippy
test
readme
@@
*/

mod cargo_task_util;
//use cargo_task_util::*;

fn main() {
    assert_eq!("CT_TEST_VAL", &std::env::var("CT_TEST_KEY").unwrap());
    ct_info!("default task is a no-op");
}
