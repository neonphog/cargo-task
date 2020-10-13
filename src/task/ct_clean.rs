use crate::*;

pub fn ct_clean(env: &cargo_task_util::CTEnv) {
    ct_info!("deleting {:?}", env.cargo_task_target);
    ct_check_fatal!(std::fs::remove_dir_all(&env.cargo_task_target));
}
