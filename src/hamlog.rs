use crate::rig::RigState;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// UNIX秒から "YYYY-MM-DD HH:MM:SS UTC" 形式の文字列に変換する。
/// (外部クレート不使用の簡易実装)
fn format_unix_secs(secs: u64) -> String {
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;

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

/// 外部(ui.rs)から呼べる公開版
pub fn format_unix_secs_pub(secs: u64) -> String {
    format_unix_secs(secs)
}

/// 外部(ui.rs)から呼べる、現在時刻の文字列を返す公開版
pub fn now_string_pub() -> String {
    format_unix_secs(now_unix_secs())
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 運用記録をローカルCSVファイルに追記する。
/// ADIF標準フィールド名に準じた項目構成(将来のADIF変換を見据えた設計)。
/// callsign / comment1 / comment2 は現時点では空欄(将来、手入力欄をGUIに追加予定)。
///
/// 出力項目: TIME_ON, TIME_OFF, FREQ, MODE, CALL, COMMENT1, COMMENT2
pub fn append_log(
    state: &RigState,
    path: &str,
    tx_started_unix: u64,
    callsign: &str,
    comment1: &str,
    comment2: &str,
    rst_sent: &str,
    rst_rcvd: &str,
) -> Result<(), String> {
    let time_on = format_unix_secs(tx_started_unix);
    let time_off = format_unix_secs(now_unix_secs());

    let freq_mhz = state
        .frequency_mhz()
        .map(|mhz| format!("{:.6}", mhz))
        .unwrap_or_else(|| "----".to_string());

    let line = format!(
        "{},{},{},{},{},{},{},{},{}\n",
        time_on, time_off, freq_mhz, state.mode, callsign, rst_sent, rst_rcvd, comment1, comment2
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

/// CSVヘッダー行(将来、ファイル新規作成時に使う想定)
pub fn csv_header() -> &'static str {
    "TIME_ON,TIME_OFF,FREQ,MODE,CALL,RST_SENT,RST_RCVD,COMMENT1,COMMENT2\n"
}

/// QsoRecord(FreeDV等、UDP経由で受信した完結済みQSO情報)から
/// 直接CSV行を追記する。RigStateのポーリングを必要としない点が
/// append_log()との違い。
pub fn append_log_from_record(
    record: &crate::log_adapter::QsoRecord,
    path: &str,
) -> Result<(), String> {
    let line = format!(
        "{},{},{},{},{},{},{},{},{}\n",
        record.time_on,
        record.time_off,
        record.freq_mhz,
        record.qso_mode,
        record.peer_call,
        record.rst_sent,
        record.rst_rcvd,
        "",
        "",
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    file.write_all(line.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}
