/*
# some help info for 'cargo help task'
@ct-help@ This suite of deps will be run in cargo-task CI. @@

@ct-task-deps@
setup
fmt-check
clippy
test
@@
*/

mod cargo_task_util;

fn main() {
    ct_info!("ci task is a no-op");
}
