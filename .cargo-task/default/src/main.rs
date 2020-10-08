/*
# this IS a default task
@ct-default@ true @@

# two other cargo-task tasks should be run before this task
@ct-task-deps@
setup
fmt-check
clippy
test
@@
*/

fn main() {}
