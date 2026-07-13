use std::fs;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

/// WSJT-XのALL.TXTを読むアダプタ
pub struct WsjtxLogAdapter {
    pub all_txt_path: String,
    pub my_call: String,
}

impl LogAdapter for WsjtxLogAdapter {
    fn latest_qso(&self) -> Option<QsoRecord> {
        read_latest_qso(&self.all_txt_path, &self.my_call)
    }

    fn name(&self) -> &'static str {
        "WSJT-X"
    }
}

fn parse_datetime(field: &str) -> String {
    let parts: Vec<&str> = field.split('_').collect();
    if parts.len() != 2 || parts[0].len() != 6 || parts[1].len() != 6 {
        return field.to_string();
    }
    let d = parts[0];
    let t = parts[1];
    format!(
        "20{}-{}-{} {}:{}:{}",
        &d[0..2], &d[2..4], &d[4..6],
        &t[0..2], &t[2..4], &t[4..6]
    )
}

fn extract_report(msg: &str) -> Option<String> {
    let body = msg.strip_prefix('R').unwrap_or(msg);
    if (body.starts_with('-') || body.starts_with('+'))
        && body.len() > 1
        && body.chars().skip(1).all(|c| c.is_ascii_digit())
    {
        Some(body.to_string())
    } else {
        None
    }
}

fn read_latest_qso(all_txt_path: &str, my_call: &str) -> Option<QsoRecord> {
    let content = fs::read_to_string(all_txt_path).ok()?;

    // 自局が関わり、かつ CQ 送信ではない行だけを対象にする
    // (CQ送信は「まだ誰とも交信していない」状態であり、相手局ではない)
    let my_lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            if !line.contains(my_call) {
                return false;
            }
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.len() < 9 {
                return false;
            }
            f[7] != "CQ"
        })
        .collect();

    if my_lines.is_empty() {
        return None;
    }

    let last_line = my_lines.last()?;
    let fields: Vec<&str> = last_line.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }
    let sender = fields[7];
    let receiver = fields[8];

    let peer_call = if sender == my_call {
        receiver.to_string()
    } else {
        sender.to_string()
    };

    let peer_lines: Vec<&&str> = my_lines
        .iter()
        .rev()
        .take_while(|line| line.contains(&peer_call))
        .collect();

    if peer_lines.is_empty() {
        return None;
    }

    let ordered: Vec<&&str> = peer_lines.iter().rev().cloned().collect();

    let freq_mhz = ordered
        .first()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("----")
        .to_string();

    // 無線機の変調方式(USB等)とは別の、通信プロトコル(FT8/FT4等)
    let qso_mode = ordered
        .first()
        .and_then(|line| line.split_whitespace().nth(3))
        .unwrap_or("----")
        .to_string();

    let time_on = ordered
        .first()
        .and_then(|line| line.split_whitespace().next())
        .map(parse_datetime)
        .unwrap_or_default();

    let time_off = ordered
        .last()
        .and_then(|line| line.split_whitespace().next())
        .map(parse_datetime)
        .unwrap_or_default();

    let mut rst_sent = String::new();
    let mut rst_rcvd = String::new();
    let mut has_73 = false;

    for line in &ordered {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 10 {
            continue;
        }
        let dir = f[2];
        let msg = f[9];

        if msg == "73" || msg == "RR73" {
            has_73 = true;
        }

        if let Some(report) = extract_report(msg) {
            if dir == "Tx" {
                rst_sent = report;
            } else {
                rst_rcvd = report;
            }
        }
    }

    let status = if has_73 {
        QsoStatus::Complete
    } else if !rst_sent.is_empty() || !rst_rcvd.is_empty() {
        QsoStatus::Incomplete
    } else {
        QsoStatus::NoResponse
    };

    Some(QsoRecord {
        peer_call,
        status: Some(status),
        rst_sent,
        rst_rcvd,
        freq_mhz,
        qso_mode,
        time_on,
        time_off,
    })
}
