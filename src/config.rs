use std::env;

pub struct Config {
    pub flrig_addr: String,
    pub poll_interval_ms: u64,
    pub activity_log_path: String,
    pub wsjtx_all_txt_path: String,
    pub fldigi_logbook_path: String,
    pub sent_to_hamlog_path: String,
    pub mmsstv_mdt_path: String,
    pub hamlog_bridge_exe_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flrig_addr: "127.0.0.1:12345".to_string(),
            poll_interval_ms: 1000,
            activity_log_path: format!("{}/ham_control_v02_activity.csv", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            wsjtx_all_txt_path: format!("{}/.local/share/WSJT-X/ALL.TXT", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            fldigi_logbook_path: format!("{}/.fldigi/logs/logbook.adif", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            sent_to_hamlog_path: format!("{}/ham_control_v02_sent_to_hamlog.txt", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            // MMSSTVはコールサイン名の.MDTファイルにログを持つ(例: JA3MBC.MDT)。
            // 呼出符号がハードコードなのは暫定。将来的にはmy_call設定と連動させたい。
            mmsstv_mdt_path: format!("{}/.wine/drive_c/MMSSTV/JA3MBC.MDT", env::var("HOME").unwrap_or_else(|_| ".".to_string())),
            // 014で実装・動作確認したWM_COPYDATA橋渡しプログラム。
            // リポジトリ同梱のhamlog_bridge/を `cargo build --target x86_64-pc-windows-gnu --release`
            // した先を既定パスにしている。
            hamlog_bridge_exe_path: format!(
                "{}/ham_control_v02/hamlog_bridge/target/x86_64-pc-windows-gnu/release/hamlog_bridge.exe",
                env::var("HOME").unwrap_or_else(|_| ".".to_string())
            ),
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

    if let Ok(path) = env::var("HAM_FLDIGI_LOGBOOK_PATH") {
        cfg.fldigi_logbook_path = path;
    }

    if let Ok(path) = env::var("HAM_SENT_TO_HAMLOG_PATH") {
        cfg.sent_to_hamlog_path = path;
    }

    if let Ok(path) = env::var("HAM_MMSSTV_MDT_PATH") {
        cfg.mmsstv_mdt_path = path;
    }

    if let Ok(path) = env::var("HAM_BRIDGE_EXE_PATH") {
        cfg.hamlog_bridge_exe_path = path;
    }

    cfg
}
