use std::env;

pub struct Config {
    pub flrig_addr: String,
    pub poll_interval_ms: u64,
    pub hamlog_addr: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flrig_addr: "127.0.0.1:12345".to_string(),
            poll_interval_ms: 1000,
            hamlog_addr: "127.0.0.1:2237".to_string(),
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

    if let Ok(addr) = env::var("HAM_HAMLOG_ADDR") {
        cfg.hamlog_addr = addr;
    }

    cfg
}
