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

/// "YYYY-MM-DD HH:MM:SS UTC"形式の文字列から、ADIF用のQSO_DATE(YYYYMMDD)と
/// TIME_ON/TIME_OFF(HHMMSS)を取り出す。想定外の形式なら空文字列を返す。
fn split_datetime_for_adif(dt: &str) -> (String, String) {
    let mut parts = dt.split_whitespace();
    let date_part = parts.next().unwrap_or("");
    let time_part = parts.next().unwrap_or("");

    let date_digits: String = date_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let time_digits: String = time_part.chars().filter(|c| c.is_ascii_digit()).collect();

    (date_digits, time_digits)
}

/// ADIFの1タグを "<NAME:LEN>VALUE" 形式で組み立てる。
fn adif_tag(name: &str, value: &str) -> String {
    format!("<{}:{}>{}", name, value.chars().count(), value)
}

/// QsoRecordから1件のADIFレコード(EOR区切り)を組み立て、ファイルへ追記する。
/// ファイルが存在しなければ、ADIFヘッダー(<EOH>まで)を先頭に書き込む。
pub fn append_adif_from_record(
    record: &crate::log_adapter::QsoRecord,
    path: &str,
) -> Result<(), String> {
    let file_exists = std::path::Path::new(path).exists();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    if !file_exists {
        let header = format!(
            "ham_control_v02 ADIF export\n<PROGRAMID:13>ham_control_v02\n<EOH>\n"
        );
        file.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
    }

    let (qso_date, time_on) = split_datetime_for_adif(&record.time_on);
    let (_, time_off) = split_datetime_for_adif(&record.time_off);

    let mut line = String::new();
    line.push_str(&adif_tag("CALL", &record.peer_call));
    if !qso_date.is_empty() {
        line.push_str(&adif_tag("QSO_DATE", &qso_date));
    }
    if !time_on.is_empty() {
        line.push_str(&adif_tag("TIME_ON", &time_on));
    }
    if !time_off.is_empty() {
        line.push_str(&adif_tag("TIME_OFF", &time_off));
    }
    line.push_str(&adif_tag("FREQ", &record.freq_mhz));
    line.push_str(&adif_tag("MODE", &record.qso_mode));
    if !record.rst_sent.is_empty() {
        line.push_str(&adif_tag("RST_SENT", &record.rst_sent));
    }
    if !record.rst_rcvd.is_empty() {
        line.push_str(&adif_tag("RST_RCVD", &record.rst_rcvd));
    }
    line.push_str("<EOR>\n");

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
