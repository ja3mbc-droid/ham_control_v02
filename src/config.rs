use std::env;

/// アプリケーション設定。環境変数で上書き可能、なければデフォルト値を使う。
pub struct Config {
    /// flrigの接続先(host:port)
    pub flrig_addr: String,
    /// ポーリング間隔(ミリ秒)
    pub poll_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flrig_addr: "127.0.0.1:12345".to_string(),
            poll_interval_ms: 1000,
        }
    }
}

/// 環境変数(HAM_FLRIG_ADDR / HAM_POLL_INTERVAL_MS)を読み、
/// 未設定ならデフォルト値を使って Config を組み立てる。
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

    cfg
}
