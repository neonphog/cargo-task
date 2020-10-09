![Crates.io](https://img.shields.io/crates/l/cargo-task)
![Crates.io](https://img.shields.io/crates/v/cargo-task)

# cargo-task

Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.

- Platform Agnostic - runs on any platform that cargo runs on.
- Zero-Dependency - the task manager itself installs almost instantly.
- Rust Task Logic - you can choose to inlude dependencies in your tasks.
- Take a look at [The Tasks in This Repo](./.cargo-task) for examples.

### Quick Start - Installation

- Install / Initialize `cargo-task`:
```shell
# Install the cargo-task cargo submodule:
cargo install cargo-task

# Initialize your rust repository with a .cargo-task directory:
cargo task ct-init

# Change to that directory:
cd .cargo-task

# Create a new task project:
cargo new --bin my-task
```

- Edit `.cargo-task/my-task/src/main.rs` to look like:
```no-compile
/*
@ct-default@ true @@
*/

// The content of this module is added by the cargo task builder.
// It contains helpers like the ct_* logging macros.
mod cargo_task_util;

fn main() {
    ct_info!("Hello World!");
}
```rust

- Test it out:
```shell
cd ..

cargo task
```

### Quick Start - Command Line API

- `cargo help task` - print out some cli help info.
- `cargo task ct-init` - initialize a repository with a .cargo-task dir.
- `cargo task ct-meta` - print out meta-information about configured tasks.
- `cargo task` - execute any "default" tasks if configured.
- `cargo task [task-name]` - execute a specific task (or list of tasks).

### AtAt (@@) cargo-task metadata

- The first `@` must be the first character on a line!
- Use double `@@` to finish the setting.

- `@ct-default@` - set to `true` to make the task a default task.
```shell
@ct-default@ true @@
```

- `@ct-help@` - specify help text to be displayed next to your task on
`cargo help task`
```shell
@ct-help@
This is a description for a task.
One line or two lines is fine.
@@
```

- `@ct-task-deps@` - whitespace delimited list of tasks that should be
run before this one.
```shell
@ct-task-deps@ task1 task2 @@
```

- `@ct-dependencies@` - reserved for when we implement light-weight
single-file tasks.
