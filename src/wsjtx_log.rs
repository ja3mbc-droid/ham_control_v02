use std::fs;

/// QSOの状態
#[derive(Debug, PartialEq)]
pub enum QsoStatus {
    Complete,    // 完全成立(73受信済み)
    Incomplete,  // 尻切れ(レポート交換あり、73未確認)
    NoResponse,  // 空振り(レポート交換なし)
}

/// ALL.TXTから、自局(my_call)が関わる直近のやり取りを解析し、
/// 相手局コールサインと状態を返す。該当データが無ければNoneを返す。
pub fn find_latest_qso(all_txt_path: &str, my_call: &str) -> Option<(String, QsoStatus)> {
    let content = fs::read_to_string(all_txt_path).ok()?;

    // 自局が関わる行を、末尾から新しい順に集める(直近のやり取りを見るため)
    let my_lines: Vec<&str> = content
        .lines()
        .filter(|line| line.contains(my_call))
        .collect();

    if my_lines.is_empty() {
        return None;
    }

    // 直近の行から、相手局コールサインを特定する
    // 行の形式: 日時 周波数 方向 モード 強度 dt 周波数(Hz) 発信局 宛先局 メッセージ
    let last_line = my_lines.last()?;
    let fields: Vec<&str> = last_line.split_whitespace().collect();
    if fields.len() < 9 {
        return None;
    }
    let sender = fields[7];
    let receiver = fields[8];

    let peer_call = if sender == my_call {
        receiver.to_string()
    } else {
        sender.to_string()
    };

    // その相手局とのやり取りだけを抽出(新しい順のまま)
    let peer_lines: Vec<&&str> = my_lines
        .iter()
        .rev()
        .take_while(|line| line.contains(&peer_call))
        .collect();

    // 73を受信していれば完全成立
    let has_73 = peer_lines.iter().any(|line| {
        line.contains(" 73") && !line.contains("RR73") && !line.trim_end().ends_with("PM74")
    }) || peer_lines.iter().any(|line| line.trim_end().ends_with(" 73"));

    if has_73 {
        return Some((peer_call, QsoStatus::Complete));
    }

    // レポート交換の有無で「尻切れ」か「空振り」かを判定する
    // レポート値の例: -06, R-17, +05 のような、数値を含むメッセージ
    let has_report_exchange = peer_lines.iter().any(|line| {
        let msg = line.split_whitespace().last().unwrap_or("");
        msg.starts_with('R') && msg.chars().skip(1).any(|c| c.is_ascii_digit())
            || (msg.starts_with('-') || msg.starts_with('+'))
                && msg.chars().skip(1).all(|c| c.is_ascii_digit())
    });

    if has_report_exchange {
        Some((peer_call, QsoStatus::Incomplete))
    } else {
        Some((peer_call, QsoStatus::NoResponse))
    }
}
