use crate::*;

/// Print user-friendly usage info.
pub fn help() {
    println!(
        r#"
# cargo task usage #

        cargo help task - this help info
             cargo task - execute all configured default cargo tasks
 cargo task [task-list] - execute a specific list of cargo tasks

# system tasks #

                ct-init - generate a '{}' directory + .gitignore
                ct-meta - print meta info about the cargo-task configuration
               ct-clean - delete the cargo-task target directory, will be
                          removed even if it matches your project target dir
"#,
        CARGO_TASK_DIR,
    );

    if env_loader::load().is_ok() {
        let env = cargo_task_util::ct_env();
        println!("# locally-defined tasks (* - default, ^ - bootstrap) #\n");

        let mut keys = env.tasks.keys().collect::<Vec<_>>();
        keys.sort();

        for task_name in keys {
            let task = env.tasks.get(task_name.as_str()).unwrap();
            let m = if task.bootstrap {
                "^"
            } else if task.default {
                "*"
            } else {
                " "
            };
            println!("{:>22}{} - {}", task.name, m, task.help);
        }

        println!();
    }
}
