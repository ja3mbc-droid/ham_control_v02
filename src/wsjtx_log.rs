use std::fs;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

/// WSJT-XのALL.TXTを読むアダプタ
pub struct WsjtxLogAdapter {
    pub all_txt_path: String,
    pub my_call: String,
}

impl WsjtxLogAdapter {
    pub fn new(all_txt_path: String, my_call: String) -> Self {
        Self {
            all_txt_path,
            my_call,
        }
    }
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

/// ALL.TXT内の自局が関わる行を、相手コールサインごとにグループ化してQsoRecordへ変換する。
/// read_latest_qso()と違い「末尾から連続する1ブロック」だけでなく、ファイル全体を
/// 相手局ごとにグループ化するため、パイルアップで複数局のやり取りが時系列上で
/// 入り乱れていても、各局ごとの交信記録を漏れなく拾える。
///
/// 戻り値は、ファイル中で各相手局が最初に登場した順。

pub fn extract_all_qsos(all_txt_path: &str, my_call: &str) -> Vec<QsoRecord> {
    const SESSION_GAP_SECS: i64 = 900; // 15分

    let content = match fs::read_to_string(all_txt_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

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

    struct Session {
        result_index: usize,
        last_date: String,
        last_secs: i64,
        completed: bool,
    }

    let mut result: Vec<QsoRecord> = Vec::new();
    let mut open_sessions: std::collections::HashMap<String, Session> =
        std::collections::HashMap::new();

    for line in &my_lines {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 10 {
            continue;
        }

        let sender = fields[7];
        let receiver = fields[8];

        let peer_call = if sender == my_call {
            receiver.to_string()
        } else {
            sender.to_string()
        };

        let freq_mhz = fields[1].to_string();
        let qso_mode = fields[3].to_string();

        let dir = fields[2];
        let msg = fields[9];

        let time_str = parse_datetime(fields[0]);
        let (date_part, secs_of_day) =
            parse_ymd_hms(&time_str).unwrap_or((String::new(), -1));

        let needs_new_session = match open_sessions.get(&peer_call) {
            None => true,
            Some(session) => {
                session.completed
                    || date_part.is_empty()
                    || session.last_date.is_empty()
                    || date_part != session.last_date
                    || secs_of_day < 0
                    || session.last_secs < 0
                    || (secs_of_day - session.last_secs).abs() > SESSION_GAP_SECS
            }
        };

        if needs_new_session {
            result.push(QsoRecord {
                peer_call: peer_call.clone(),
                status: Some(QsoStatus::NoResponse),
                rst_sent: String::new(),
                rst_rcvd: String::new(),
                freq_mhz,
                qso_mode,
                time_on: time_str.clone(),
                time_off: time_str.clone(),
            });

            open_sessions.insert(
                peer_call.clone(),
                Session {
                    result_index: result.len() - 1,
                    last_date: date_part.clone(),
                    last_secs: secs_of_day,
                    completed: false,
                },
            );
        }

        let session = open_sessions.get_mut(&peer_call).unwrap();
        let record = &mut result[session.result_index];

        record.time_off = time_str.clone();

        if msg == "73" || msg == "RR73" {
            session.completed = true;
            record.status = Some(QsoStatus::Complete);
        }

        if let Some(report) = extract_report(msg) {
            if dir == "Tx" {
                record.rst_sent = report;
            } else {
                record.rst_rcvd = report;
            }

            if record.status != Some(QsoStatus::Complete) {
                record.status = Some(QsoStatus::Incomplete);
            }
        }

        if !date_part.is_empty() {
            session.last_date = date_part;
        }
        session.last_secs = secs_of_day;
    }

    result
}

fn parse_ymd_hms(dt: &str) -> Option<(String, i64)> {
    let mut parts = dt.splitn(2, ' ');
    let date = parts.next()?.to_string();
    let time = parts.next()?;

    let t: Vec<&str> = time.split(':').collect();

    if t.len() != 3 {
        return None;
    }

    let h: i64 = t[0].parse().ok()?;
    let m: i64 = t[1].parse().ok()?;
    let s: i64 = t[2].parse().ok()?;

    Some((date, h * 3600 + m * 60 + s))
}
