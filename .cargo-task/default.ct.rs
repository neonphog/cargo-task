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

use cargo_task_util::*;

fn main() {
    assert_eq!("MY_TEST_VAL", &std::env::var("MY_TEST_KEY").unwrap());
    ct_info!("default task is a no-op");
}
