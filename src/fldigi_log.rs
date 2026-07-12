use std::fs;

/// fldigiのlogbook.adif(ADIF形式)から、最新のQSO 1件を抽出する。
/// 土台段階の実装: 実データが手に入り次第、wsjtx_log.rs の時と同様に
/// 実際のfldigi出力を確認し、フィールド名や区切りのズレを検証・修正する。
pub struct FldigiQsoInfo {
    pub call: String,
    pub qso_date: String,
    pub time_on: String,
    pub freq_mhz: String,
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
}

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

/// logbook.adif から最新のQSO(最後のレコード)を読み取る。
pub fn find_latest_qso(adif_path: &str) -> Option<FldigiQsoInfo> {
    let content = fs::read_to_string(adif_path).ok()?;

    // ヘッダー(<EOH>まで)を除いた本文を、<EOR>区切りでレコードに分割
    let body = content.split("<EOH>").nth(1)?;
    let records: Vec<&str> = body.split("<EOR>").filter(|r| !r.trim().is_empty()).collect();

    let last_record = records.last()?;
    let fields = parse_adif_record(last_record);

    Some(FldigiQsoInfo {
        call: fields.get("CALL").cloned().unwrap_or_default(),
        qso_date: fields.get("QSO_DATE").cloned().unwrap_or_default(),
        time_on: fields.get("TIME_ON").cloned().unwrap_or_default(),
        freq_mhz: fields.get("FREQ").cloned().unwrap_or_default(),
        mode: fields.get("MODE").cloned().unwrap_or_default(),
        rst_sent: fields.get("RST_SENT").cloned().unwrap_or_default(),
        rst_rcvd: fields.get("RST_RCVD").cloned().unwrap_or_default(),
    })
}
