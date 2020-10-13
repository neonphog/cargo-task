use crate::*;

pub fn ct_meta(env: &cargo_task_util::CTEnv) {
    ct_info!("print full cargo-task metadata");
    println!("{:#?}", env);
}
