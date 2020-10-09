/*
# this IS a default task
@ct-default@ true @@

# some help info for 'cargo help task'
@ct-help@ This meta default task just ensures other tasks run. @@

# two other cargo-task tasks should be run before this task
@ct-task-deps@
setup
fmt-check
clippy
test
@@
*/

fn main() {}
