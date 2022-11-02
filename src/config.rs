use std::{env, path::PathBuf};

pub fn get_config_path() -> PathBuf {
    let xdg_config = env::var("XFG_CONFIG_HOME").map(PathBuf::from);
    let user_profile = env::var("USERPROFILE").map(|path| PathBuf::from(path).join(".config"));

    match (xdg_config, user_profile) {
        (Ok(path), _) => path,
        (_, Ok(path)) => path,
        _ => panic!("Could not find config directory"),
    }
}
