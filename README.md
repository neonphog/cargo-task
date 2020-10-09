![Crates.io](https://img.shields.io/crates/l/cargo-task)
![Crates.io](https://img.shields.io/crates/v/cargo-task)

# cargo-task

Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.

### Usage

- Install the cargo subcommand:
```shell
cargo install cargo-task
```

- Initialize your rust repository with a .cargo-task directory:
```shell
cargo task ct-init
```

- Change to that directory:
```shell
cd .cargo-task
```

- Create a new task project:
```shell
cargo new --bin my-task
```

- Edit `.cargo-task/my-task/src/main.rs` to look like:
```shell
/*
@ct-default@ true @@
*/

mod cargo_task_util;

fn main() {
    ct_info!("Hello World!");
}
```

- Return to your root directory:
```shell
cd ..
```

- Run 'cargo task
```shell
cargo task
```
