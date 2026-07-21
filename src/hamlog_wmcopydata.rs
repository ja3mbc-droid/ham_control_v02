use crate::log_adapter::{QsoRecord, QsoStatus};
use std::io::Write;
use std::process::{Command, Stdio};

/// 相手コールサインが日本の局かどうかを判定する。
/// 対象: JA〜JS(2文字目がA〜S)の一般的な日本の割当プレフィックス、
/// および7J/7K/7L/7M/7N・8J/8K/8L/8M/8N(記念局等に使われる数字接頭)。
/// JT〜JV(モンゴル)・JW〜JX(ノルウェー)・JY(ヨルダン)・JZ(インドネシア)は
/// 日本ではないため対象外(2文字目をA〜Sに限定することで除外している)。
/// 完全な正確性は保証しないが、実運用上の大半のケースをカバーする簡易判定。
fn is_japanese_callsign(call: &str) -> bool {
    let call = call.trim().to_uppercase();
    let chars: Vec<char> = call.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    if chars[0] == 'J' && ('A'..='S').contains(&chars[1]) {
        return true;
    }
    if chars[0].is_ascii_digit() && ('J'..='N').contains(&chars[1]) {
        return true;
    }
    false
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

/// UTC日時(年,月,日,時,分)にadd_hours時間を加算し、日付の繰り上がり/繰り下がりを
/// 考慮した現地時刻を計算する(JST変換=+9のために使用)。
fn add_hours(year: i32, month: u32, day: u32, hour: u32, minute: u32, add_hours: i32) -> (i32, u32, u32, u32, u32) {
    let mut total_minutes = hour as i64 * 60 + minute as i64 + add_hours as i64 * 60;
    let mut day = day as i64;
    let mut month = month as i64;
    let mut year = year as i64;

    while total_minutes >= 24 * 60 {
        total_minutes -= 24 * 60;
        day += 1;
        let dim = days_in_month(year as i32, month as u32) as i64;
        if day > dim {
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    while total_minutes < 0 {
        total_minutes += 24 * 60;
        day -= 1;
        if day < 1 {
            month -= 1;
            if month < 1 {
                month = 12;
                year -= 1;
            }
            day = days_in_month(year as i32, month as u32) as i64;
        }
    }

    let hour = (total_minutes / 60) as u32;
    let minute = (total_minutes % 60) as u32;
    (year as i32, month as u32, day as u32, hour, minute)
}

/// record.time_on("YYYY-MM-DD HH:MM:SS"形式、UTC基準、wsjtx_log.rs/fldigi_log.rs/
/// freedv_log.rs/mmsstv_log.rsで共通化済み)を、HAMLOGのDate欄("26/07/21"形式)・
/// Time欄("13:10U"/"22:10J"形式、末尾J/Uでタイムゾーンを示す)に変換する。
/// 相手が日本の局(is_japanese_callsignでtrue)ならJST(+9h)に変換してJを付け、
/// それ以外はUTCのままUを付ける。パースできない場合はNoneを返し、
/// 呼び出し側は空欄のまま送る(HAMLOG既定値に任せる)。
fn time_on_to_hamlog_date_time(time_on: &str, peer_call: &str) -> Option<(String, String)> {
    let mut parts = time_on.splitn(2, ' ');
    let date_part = parts.next()?;
    let time_part = parts.next()?;

    let date_fields: Vec<&str> = date_part.split('-').collect();
    if date_fields.len() != 3 {
        return None;
    }
    let year: i32 = date_fields[0].parse().ok()?;
    let month: u32 = date_fields[1].parse().ok()?;
    let day: u32 = date_fields[2].parse().ok()?;

    let time_fields: Vec<&str> = time_part.split(':').collect();
    if time_fields.len() < 2 {
        return None;
    }
    let hour: u32 = time_fields[0].parse().ok()?;
    let minute: u32 = time_fields[1].parse().ok()?;

    let (y, mo, d, h, mi) = if is_japanese_callsign(peer_call) {
        let (y, mo, d, h, mi) = add_hours(year, month, day, hour, minute, 9);
        (y, mo, d, h, mi)
    } else {
        (year, month, day, hour, minute)
    };
    let tz_suffix = if is_japanese_callsign(peer_call) { "J" } else { "U" };

    let yy = y % 100;
    let hamlog_date = format!("{:02}/{:02}/{:02}", yy, mo, d);
    let hamlog_time = format!("{:02}:{:02}{}", h, mi, tz_suffix);
    Some((hamlog_date, hamlog_time))
}

/// dwData=15で送る「コールサイン〜Remarks2 + チェックボックス」16行の並び。
/// 014の実機テスト(L01〜L16を送って確認)で確定した並び:
///   1行目: チェックボックス関連(未使用、空)
///   2〜15行目: Call, Date, Time, His, My, Freq, Mode, Code, G.L, QSL,
///              HisName, QTH, Remarks1, Remarks2
///   16行目: チェックボックス関連(未使用、空)
fn qso_record_to_16lines(record: &QsoRecord) -> String {
    let remarks1 = match record.status {
        // xdotool時代の運用を踏襲: 73未確認(尻切れ)QSOはRemarks1に印を残す
        Some(QsoStatus::Incomplete) => "[73未確認]",
        _ => "",
    };

    let (date_str, time_str) = time_on_to_hamlog_date_time(&record.time_on, &record.peer_call)
        .unwrap_or_default();

    let lines: [&str; 16] = [
        "",                    // 1: チェックボックス(未使用)
        &record.peer_call,     // 2: Call
        &date_str,             // 3: Date
        &time_str,             // 4: Time
        &record.rst_sent,      // 5: His
        &record.rst_rcvd,      // 6: My
        &record.freq_mhz,      // 7: Freq
        &record.qso_mode,      // 8: Mode
        "",                    // 9: Code
        "",                    // 10: G.L
        "",                    // 11: QSL
        "",                    // 12: HisName
        "",                    // 13: QTH
        remarks1,              // 14: Remarks1
        "",                    // 15: Remarks2
        "",                    // 16: チェックボックス(未使用)
    ];

    lines.join("\n")
}

/// hamlog_bridge.exe(Wine上で動くWM_COPYDATA送信専用プログラム)を1回呼び出す。
/// payloadは標準入力(UTF-8)として渡す。dwDataはフラグをor済みの10進数値。
fn run_bridge(bridge_exe_path: &str, dw_data: u32, payload: &str) -> Result<String, String> {
    let mut child = Command::new("wine")
        .arg(bridge_exe_path)
        .arg(dw_data.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("wine {}の起動に失敗しました: {}", bridge_exe_path, e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("hamlog_bridgeへの入力書き込みに失敗しました: {}", e))?;
        // stdinをここでdropしてEOFを送る(hamlog_bridge側はread_to_stringで待っている)
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("hamlog_bridgeの終了待ちに失敗しました: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("hamlog_bridge(dwData={})が失敗しました: {}", dw_data, stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// dwData=16(クリア)→dwData=15(全項目送信)の2段階を実行し、
/// LOG-[A]画面にQSO情報を流し込む。
///
/// 意図的にdwData=18(保存)はここでは呼ばない。014のインシデント
/// (テスト残骸が実運用ログに混入した件)を踏まえ、保存は運用者が
/// 画面を目視確認したうえで手動でSaveボタンを押す運用とする。
pub fn fill_hamlog_window(bridge_exe_path: &str, record: &QsoRecord) -> Result<(), String> {
    // dwData=16: 入力バッファのクリア(前回の残骸が混ざらないよう必ず先に実行)
    run_bridge(bridge_exe_path, 16, "")?;

    // dwData=15: 16行まとめて送信
    let payload = qso_record_to_16lines(record);
    run_bridge(bridge_exe_path, 15, &payload)?;

    Ok(())
}

/// Date(dwData=2)・Time(dwData=3)だけを個別に狙い撃ちで再送信する。
///
/// HAMLOG本体は、Call欄でEnterされた時点で「新規コールサイン処理」を発火し、
/// 現在日時・ユーザー名・QTH等を自動的に取得してDate/Time欄を上書きしてしまう
/// (2026-07-21判明)。そのため、fill_hamlog_window()で一旦全項目を送っても、
/// 運用者がCall欄で確認のためEnterを押すとDate/Timeが現在日時に戻ってしまう。
///
/// この関数は、Saveを押す直前に呼び出す想定。dwData=15(全項目送信)ではなく
/// dwData=1〜14の個別項目コマンドのうちDate(2)とTime(3)だけを送るため、
/// 既に入力済みのCall等の他の項目には一切触れない。
pub fn resend_date_time(bridge_exe_path: &str, record: &QsoRecord) -> Result<(), String> {
    let (date_str, time_str) = time_on_to_hamlog_date_time(&record.time_on, &record.peer_call)
        .ok_or_else(|| "QSO時刻を解釈できませんでした(time_onが空、または想定外の形式)".to_string())?;

    run_bridge(bridge_exe_path, 2, &date_str)?; // dwData=2: Date
    run_bridge(bridge_exe_path, 3, &time_str)?; // dwData=3: Time

    Ok(())
}
