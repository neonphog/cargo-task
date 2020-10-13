![Crates.io](https://img.shields.io/crates/l/cargo-task)
![Crates.io](https://img.shields.io/crates/v/cargo-task)

# cargo-task

Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.

- Platform Agnostic - runs on any platform that cargo runs on.
- Zero-Dependency - the task manager itself installs almost instantly.
- Rust Task Logic - stop writing separate bash and powershell scripts.
- Take a look at the ['.cargo-task' directory](./.cargo-task) in this repo for examples.

```shell
cargo install -f cargo-task
cargo help task
```

### Creating `cargo-task` automation tasks.

```shell
cargo task ct-init
cd .cargo-task
cargo new --bin my-task
cd ..
cargo task my-task
```

- `cargo task ct-init` - creates the `.cargo-task` directory and `.gitignore`.
- `cargo task my-task` - runs the crate named `my-task` defined in the `.cargo-task` directory.

It's that simple!

### Customizing how tasks are executed.

`cargo-task` uses a metadata format called AtAt - because it uses `@` signs:

```rust
/*
@ct-default@ true @@
@ct-task-deps@
one
two
@@
*/
```

Some things to know about AtAt:
- protocol: `@key@ value @@`.
- the first `@` for the key must be the first character on a line.
- the value is terminated by a two ats, "`@@`".
- the value can contain newlines or be on a single line.
- you probably want it in a rust comment block : )

These directives will be read from your `main.rs` file when parsing the
`.cargo-task` crates.

#### Default tasks.

```rust
/*
@ct-default@ true @@
*/
```

Default tasks will be executed if the task list is empty on `cargo task`
invocations.

#### Bootstrap tasks.

```rust
/*
@ct-bootstrap@ true @@
*/
```

Bootstrap tasks will *always* be executed before any task-list tasks.
Also, the cargo-task metadata will be reloaded after bootstrap tasks
are executed. You can use this to download / install / configure
additional tasks.

#### Task dependencies.

```rust
/*
@ct-task-deps@
my-first-dependency
my-second-dependency
@@
*/
```

A whitespace delimited list of tasks that must be run prior to the current
task. Can be on a single line or multiple lines.

### The magic `cargo_task_util` module.

- [cargo_task_util on docs.rs](https://docs.rs/cargo-task/latest/cargo_task/cargo_task_util/index.html)

This module will be available at the root of your crate during build time.
To include it, simply add a `mod` directive in your `main.rs` file.

```rust
/*
@ct-default@ true @@
*/

mod cargo_task_util;
use cargo_task_util::*;

fn main() {
    // cargo task metadata env helper
    let env = ct_env();

    // print out some cool cargo-task metadata
    // (this does the same thing as `cargo task ct-meta`)
    println!("{:#?}", env);

    // also includes cargo-task special log helpers
    ct_warn!("ahh, a warning! {:?}", std::time::SystemTime::now());
}
```

### Exporting environment variables to configure other tasks.

`cargo_task_util::CTEnv` also includes a utility for exporting environment
variables.

If you just use the rust `std::env::set_var` function, the variable will
be set for the current task execution, but no other tasks will see it.

Instead you can use `cargo_task_util::CTEnv::set_env` function.

You probably want to do this in a "bootstrap" task so it is available
to other tasks that are run later.

```rust
/*
@ct-bootstrap@ true @@
*/

mod cargo_task_util;
use cargo_task_util::*;

fn main() {
    // cargo task metadata env helper
    let env = ct_env();

    // set a variable that will be available in other tasks.
    env.set_env("MY_VARIABLE", "MY_VALUE");
}
```
