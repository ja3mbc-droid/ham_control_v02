use crate::log_adapter::{QsoRecord, QsoStatus};
use std::io::Write;
use std::process::{Command, Stdio};

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

    let lines: [&str; 16] = [
        "",                    // 1: チェックボックス(未使用)
        &record.peer_call,     // 2: Call
        "",                    // 3: Date(HAMLOG側の既定値に任せる)
        "",                    // 4: Time(同上)
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
