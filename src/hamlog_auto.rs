use crate::log_adapter::QsoRecord;
use std::process::Command;

/// HAMLOG(Wine上のTurbo HAMLOG/Win)の新規QSO入力画面(LOG-[A])のウィンドウ名に
/// 含まれる特徴的な部分文字列。実機で`xdotool getactivewindow getwindowname`を
/// 実行して確認した実際のタイトルは " ＬＯＧ-[Ａ]　　　"(全角文字・全角スペース
/// を含む)。全角の"["「]"部分はASCIIの角括弧ではなくASCII括弧そのままなので、
/// xdotoolのregex検索で誤動作しないよう、括弧を含まない安全な部分文字列
/// "ＬＯＧ-"(全角LOG+ハイフン)だけを検索キーとして使う。
const HAMLOG_WINDOW_NAME_FRAGMENT: &str = "ＬＯＧ-";

/// LOG-[A]画面のTabキー移動順(Call欄を0番目として):
/// Call(0) → Date(1) → Time(2) → His=RST RCVD(3) → My=RST SENT(4) → Freq(5)
/// → Mode(6) → Code(7) → G.L(8) → QSL(9) → His Name(10) → QTH(11)
/// → Remarks1(12) → Remarks2(13) → Save(14)
///
/// 実機で確認済みのTab移動順(2026-07-18時点)。HAMLOG側の画面レイアウトが
/// 変わった場合はここを直す必要がある。
struct TabPositions;
impl TabPositions {
    const DATE: u32 = 1;
    const TIME: u32 = 1; // Date直後、Dateからさらに1回
    const HIS_RST_RCVD: u32 = 1; // Time直後
    const MY_RST_SENT: u32 = 1; // His直後
    const FREQ: u32 = 1; // My直後
    const MODE: u32 = 1; // Freq直後
    // Mode → Code → G.L → QSL → His Name → QTH → Remarks1 は4回Tabをスキップしてから
    // QTHまで進み、そこからさらに1回でRemarks1
    const SKIP_CODE_GL_QSL_HISNAME: u32 = 4; // Mode直後、Code/G.L/QSL/HisNameをスキップ
    const QTH: u32 = 1;
    const REMARKS1: u32 = 1;
    const REMARKS2: u32 = 1;
    // Remarks2からSaveへはさらに1回
    const SAVE: u32 = 1;
}

/// HAMLOGのLOG-[A]画面へQSO情報を自動入力する。
///
/// 重要: この関数は**Save欄にフォーカスが乗った状態で停止**し、Enter(保存確定)は
/// 自分では押さない。理由は、自動入力された内容(特にHAMLOGがCall欄のReturnで
/// 自動展開する局データベース情報や、RST等)を運用者が目視確認し、必要なら
/// Shift+Tabで戻って修正・加筆してから、最終的に運用者自身がEnterを押して
/// 保存する、という運用フローを守るため(2026-07-18、ユーザーとの合意による)。
///
/// 前提: HAMLOGのLOG-[A]入力画面が既に開いていること。開いていない、または
/// 複数開いている場合はエラーを返す。
pub fn send_qso_to_hamlog(record: &QsoRecord, comment1: &str) -> Result<(), String> {
    let window_id = find_hamlog_window()?;

    // ウィンドウをアクティブ化し、描画が追いつくまで少し待つ(--syncで同期待ちするが
    // 念のためsleepも入れる)
    run_xdotool(&["windowactivate", "--sync", &window_id])?;
    sleep_ms(150);

    // Call欄には既にフォーカスがある前提(LOG-[A]を開いた直後の初期状態)。
    // 念のため全選択してから上書きする(前回入力が残っている場合の対策)。
    run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "ctrl+a"])?;
    type_text(&window_id, &record.peer_call)?;

    // Call欄でEnter → HAMLOGが局データベース(交信履歴 or 登録情報)を自動展開する。
    // 展開処理の完了を待つため少し長めにsleepする。
    run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "Return"])?;
    sleep_ms(400);

    // Date欄はスキップ(HAMLOG側で今日の日付が入っている想定。過去日の交信を
    // 転記する場合は、この後の手動確認・修正フェーズで運用者が直す)
    tab(&window_id, TabPositions::DATE)?;

    // Time欄: WSJT-Xのtime_on("YYYY-MM-DD HH:MM:SS")からHHMM部分を取り出して入力
    tab(&window_id, TabPositions::TIME)?;
    if let Some(hhmm) = extract_hhmm(&record.time_on) {
        type_text(&window_id, &hhmm)?;
    }

    // His = RST RCVD
    tab(&window_id, TabPositions::HIS_RST_RCVD)?;
    if !record.rst_rcvd.is_empty() {
        run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "ctrl+a"])?;
        type_text(&window_id, &record.rst_rcvd)?;
    }

    // My = RST SENT
    tab(&window_id, TabPositions::MY_RST_SENT)?;
    if !record.rst_sent.is_empty() {
        run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "ctrl+a"])?;
        type_text(&window_id, &record.rst_sent)?;
    }

    // Freq
    tab(&window_id, TabPositions::FREQ)?;
    if !record.freq_mhz.is_empty() {
        run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "ctrl+a"])?;
        type_text(&window_id, &record.freq_mhz)?;
    }

    // Mode
    tab(&window_id, TabPositions::MODE)?;
    if !record.qso_mode.is_empty() {
        run_xdotool(&["key", "--window", &window_id, "--clearmodifiers", "ctrl+a"])?;
        type_text(&window_id, &record.qso_mode)?;
    }

    // Code, G.L, QSL, His Nameをスキップ(Call欄のReturnで自動展開済みの想定、
    // 上書きしない)
    tab(&window_id, TabPositions::SKIP_CODE_GL_QSL_HISNAME)?;

    // QTHもスキップ
    tab(&window_id, TabPositions::QTH)?;

    // Remarks1: 尻切れ等のコメント(comment1が空なら何も入力せずTabだけ進める)
    tab(&window_id, TabPositions::REMARKS1)?;
    if !comment1.is_empty() {
        type_text(&window_id, comment1)?;
    }

    // Remarks2はスキップ
    tab(&window_id, TabPositions::REMARKS2)?;

    // Save欄まで進めて停止。Enterは押さない。
    tab(&window_id, TabPositions::SAVE)?;

    Ok(())
}

/// xdotoolでLOG-[A]画面のウィンドウIDを1つ探す。
/// 見つからない場合、または複数見つかった場合はエラーを返す
/// (複数ある場合にどれへ送るべきか自動判断すると事故のもとになるため)。
fn find_hamlog_window() -> Result<String, String> {
    let output = Command::new("xdotool")
        .args(["search", "--name", HAMLOG_WINDOW_NAME_FRAGMENT])
        .output()
        .map_err(|e| format!("xdotool search failed to run: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "xdotool search failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let ids: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    match ids.len() {
        0 => Err("HAMLOGのLOG-[A]画面が見つかりません。先にHAMLOGで新規QSO入力画面を開いてください。".to_string()),
        1 => Ok(ids[0].clone()),
        _ => Err(format!(
            "HAMLOGのLOG-[A]画面が複数({}個)見つかりました。1つだけ開いた状態にしてください。",
            ids.len()
        )),
    }
}

fn tab(window_id: &str, count: u32) -> Result<(), String> {
    for _ in 0..count {
        run_xdotool(&["key", "--window", window_id, "--clearmodifiers", "Tab"])?;
    }
    Ok(())
}

fn type_text(window_id: &str, text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    run_xdotool(&["type", "--window", window_id, "--clearmodifiers", text])
}

fn run_xdotool(args: &[&str]) -> Result<(), String> {
    let status = Command::new("xdotool")
        .args(args)
        .status()
        .map_err(|e| format!("xdotool実行に失敗しました: {}", e))?;

    if !status.success() {
        return Err(format!("xdotoolがエラー終了しました: {:?}", args));
    }
    Ok(())
}

fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

/// "YYYY-MM-DD HH:MM:SS"形式の文字列から"HHMM"部分を取り出す。
/// フォーマットが想定と違う場合はNoneを返す(呼び出し側でTime欄への入力を
/// スキップする)。
fn extract_hhmm(time_on: &str) -> Option<String> {
    let time_part = time_on.split(' ').nth(1)?;
    let parts: Vec<&str> = time_part.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    Some(format!("{}{}", parts[0], parts[1]))
}
