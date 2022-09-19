use env_logger::Env;

pub mod data;
pub mod algorithm;
pub mod utils;

/// Initialize logging
pub fn init_logging() {
    //initializing the logger
    let env = Env::default()
        .filter_or("TRAILSCOUT_LOG_LEVEL", "debug")
        .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::try_init_from_env(env).ok();
}