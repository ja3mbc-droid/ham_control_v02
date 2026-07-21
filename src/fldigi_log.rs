use std::fs;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

/// fldigi logbook.adif を読むAdapter
pub struct FldigiLogAdapter {
    pub adif_path: String,
}

impl FldigiLogAdapter {
    pub fn new(adif_path: String) -> Self {
        Self {
            adif_path,
        }
    }
}

impl LogAdapter for FldigiLogAdapter {
    fn latest_qso(&self) -> Option<QsoRecord> {
        println!("[Fldigi] reading {}", self.adif_path);
        find_latest_qso(&self.adif_path)
    }

    fn name(&self) -> &'static str {
        "fldigi"
    }
}

/// fldigiのlogbook.adif(ADIF形式)から、最新のQSO 1件を抽出する。
/// 土台段階の実装: 実データが手に入り次第、wsjtx_log.rs の時と同様に
/// 実際のfldigi出力を確認し、フィールド名や区切りのズレを検証・修正する。
/// ADIFの1タグ("<CALL:6>JA3MBC"のような形式)から、
/// タグ名と値を取り出す簡易パーサ。
fn parse_adif_record(record: &str) -> std::collections::HashMap<String, String> {
    let mut fields = std::collections::HashMap::new();
    let mut rest = record;

    while let Some(start) = rest.find('<') {
        let after_open = &rest[start + 1..];
        let Some(end) = after_open.find('>') else { break; };
        let tag_content = &after_open[..end]; // 例: "CALL:6"
        let after_tag = &after_open[end + 1..];

        let parts: Vec<&str> = tag_content.splitn(2, ':').collect();
        if parts.len() != 2 {
            rest = after_tag;
            continue;
        }
        let name = parts[0].to_uppercase();
        let Ok(len) = parts[1].parse::<usize>() else {
            rest = after_tag;
            continue;
        };
        if after_tag.len() < len {
            break;
        }
        let value = after_tag[..len].to_string();
        fields.insert(name, value);
        rest = &after_tag[len..];
    }

    fields
}

/// logbook.adif から全QSOをファイル中の記録順(古い→新しい)で読み取る。
/// find_latest_qso()と、GUIの一覧表示用recent_fldigi_qsos()の両方から使う共通処理。
pub fn find_all_qsos(adif_path: &str) -> Vec<QsoRecord> {
    let content = match fs::read_to_string(adif_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // ヘッダー(<EOH>まで)を除いた本文を、<EOR>区切りでレコードに分割
    let body = match content.split("<EOH>").nth(1) {
        Some(b) => b,
        None => return Vec::new(),
    };
    let records: Vec<&str> = body.split("<EOR>").filter(|r| !r.trim().is_empty()).collect();

    records
        .iter()
        .map(|record| {
            let fields = parse_adif_record(record);
            // QSO_DATE(YYYYMMDD)とTIME_ON/TIME_OFF(HHMMSS)を、
            // wsjtx_log.rs/mmsstv_log.rsと揃えた"YYYY-MM-DD HH:MM:SS"形式に正規化する。
            // hamlog_wmcopydata.rs側で全ソース共通のフォーマットとしてパースするため。
            let qso_date = fields.get("QSO_DATE").cloned().unwrap_or_default();
            let format_dt = |date: &str, time: &str| -> String {
                if date.len() == 8 && time.len() >= 4 {
                    let hh = &time[0..2];
                    let mm = &time[2..4.min(time.len())];
                    let ss = if time.len() >= 6 { &time[4..6] } else { "00" };
                    format!(
                        "{}-{}-{} {}:{}:{}",
                        &date[0..4], &date[4..6], &date[6..8], hh, mm, ss
                    )
                } else {
                    time.to_string()
                }
            };
            let time_on_raw = fields.get("TIME_ON").cloned().unwrap_or_default();
            let time_off_raw = fields.get("TIME_OFF").cloned().unwrap_or_default();

            QsoRecord {
                peer_call: fields.get("CALL").cloned().unwrap_or_default(),
                status: Some(QsoStatus::Complete),
                time_on: format_dt(&qso_date, &time_on_raw),
                freq_mhz: fields.get("FREQ").cloned().unwrap_or_default(),
                qso_mode: fields.get("MODE").cloned().unwrap_or_default(),
                rst_sent: fields.get("RST_SENT").cloned().unwrap_or_default(),
                rst_rcvd: fields.get("RST_RCVD").cloned().unwrap_or_default(),
                time_off: format_dt(&qso_date, &time_off_raw),
            }
        })
        .filter(|r| !r.peer_call.is_empty())
        .collect()
}

/// logbook.adif から最新のQSO(最後のレコード)を読み取る。
pub fn find_latest_qso(adif_path: &str) -> Option<QsoRecord> {
    let records = find_all_qsos(adif_path);
    let last = records.last()?.clone();

    println!(
        "[Fldigi] latest CALL={:?} MODE={:?} FREQ={:?}",
        last.peer_call, last.qso_mode, last.freq_mhz
    );

    Some(last)
}
