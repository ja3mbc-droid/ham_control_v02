use crate::rig::RigState;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 簡易的な UTC 日時文字列(外部クレート不使用)
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;

    // 1970-01-01 からの日数を年月日に変換(簡易実装)
    let mut year = 1970;
    let mut days_left = days as i64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let year_days = if leap { 366 } else { 365 };
        if days_left < year_days {
            break;
        }
        days_left -= year_days;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [31, if leap {29} else {28}, 31,30,31,30,31,31,30,31,30,31];
    let mut month = 1;
    for &md in month_days.iter() {
        if days_left < md { break; }
        days_left -= md;
        month += 1;
    }
    let day = days_left + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", year, month, day, h, m, s)
}

/// 運用記録をローカルファイルに追記する。
/// tx_seconds: 直前の送信(TX)が何秒間続いたか
pub fn append_log(state: &RigState, path: &str, tx_seconds: f64) -> Result<(), String> {
    let freq_mhz = state
        .frequency_mhz()
        .map(|mhz| format!("{:.6}", mhz))
        .unwrap_or_else(|| "----".to_string());

    let line = format!(
        "{},{},{},{:.1}\n",
        now_string(),
        freq_mhz,
        state.mode,
        tx_seconds
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}
