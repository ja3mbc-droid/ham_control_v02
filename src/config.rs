use std::env;

pub struct Config {
    pub flrig_addr: String,
    pub poll_interval_ms: u64,
    pub activity_log_path: String,
    pub wsjtx_all_txt_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flrig_addr: "127.0.0.1:12345".to_string(),
            poll_interval_ms: 1000,
            activity_log_path: format!("{}/ham_control_v02_activity.csv", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            wsjtx_all_txt_path: format!("{}/.local/share/WSJT-X/ALL.TXT", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
        }
    }
}

pub fn load() -> Config {
    let mut cfg = Config::default();

    if let Ok(addr) = env::var("HAM_FLRIG_ADDR") {
        cfg.flrig_addr = addr;
    }

    if let Ok(interval) = env::var("HAM_POLL_INTERVAL_MS") {
        if let Ok(ms) = interval.parse::<u64>() {
            cfg.poll_interval_ms = ms;
        }
    }

    if let Ok(path) = env::var("HAM_ACTIVITY_LOG_PATH") {
        cfg.activity_log_path = path;
    }

    if let Ok(path) = env::var("HAM_WSJTX_ALL_TXT_PATH") {
        cfg.wsjtx_all_txt_path = path;
    }

    cfg
}
