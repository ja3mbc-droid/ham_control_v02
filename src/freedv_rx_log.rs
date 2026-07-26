use std::fs;

/// FreeDV(freedv-gui)が「Log heard callsigns to a CSV file」機能で書き出す
/// freedv_rx_log.csv の1行に対応する、受信ログ専用のレコード。
///
/// QsoRecord(log_adapter.rs)とは意図的に完全分離している。QSO(実際に交信が
/// 成立した局)とStations Heard(聞こえただけの局。交信の有無を問わない)は
/// 性質が異なるデータであり、混在させるとHAMLOG送信ボタンの導線に紛れ込む
/// 事故(交信していない局を誤ってHAMLOGへ送ってしまう)につながるため。
///
/// フォーマットはFreeDV公式リポジトリ(drowe67/freedv-gui)のsrc/main.cpp
/// (restoreCallsignListFromCsv_)を実機確認の代わりに参照して特定した:
/// ヘッダー行1行 + "date,time,callsign,mode,frequency_hz,snr_db" のCSV行。
/// date/timeの実際の書式はFreeDV側のCsvReporter実装依存で未確認のため、
/// 生の文字列としてそのまま保持し、この場では正規化しない(壊さない)。
#[derive(Debug, Clone, Default)]
pub struct RxStationRecord {
    pub date: String,
    pub time: String,
    pub callsign: String,
    pub mode: String,
    pub frequency_hz: u64,
    pub snr_db: i32,
}

/// CSVの1行("date,time,callsign,mode,frequency_hz,snr_db")をパースする。
/// 列数が足りない行、frequency_hz/snr_dbが数値でない行はNoneを返し、
/// 呼び出し元でスキップされる(1行の破損で全体を巻き込まない)。
fn parse_csv_line(line: &str) -> Option<RxStationRecord> {
    let mut cols = line.splitn(6, ',');
    let date = cols.next()?.trim().to_string();
    let time = cols.next()?.trim().to_string();
    let callsign = cols.next()?.trim().to_string();
    let mode = cols.next()?.trim().to_string();
    let frequency_hz: u64 = cols.next()?.trim().parse().ok()?;
    let snr_db: i32 = cols.next()?.trim().parse().ok()?;

    if callsign.is_empty() {
        return None;
    }

    Some(RxStationRecord {
        date,
        time,
        callsign,
        mode,
        frequency_hz,
        snr_db,
    })
}

/// freedv_rx_log.csv から全レコードを、ファイル中の記録順(古い→新しい)で読み取る。
/// wsjtx_log.rs/fldigi_log.rs/mmsstv_log.rsのfind_all_qsos()と揃えた命名・
/// シグネチャにしてあるが、戻り値はQsoRecordではなくRxStationRecordであり、
/// QSO処理系統(LogAdapter/LogManager)には一切触れない。
pub fn find_all_stations(csv_path: &str) -> Vec<RxStationRecord> {
    let content = match fs::read_to_string(csv_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .skip(1) // 1行目はヘッダー("date,time,callsign,mode,frequency_hz,snr_db")
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_csv_line)
        .collect()
}

/// Stations Heard表示用。新しい順(最新が先頭)で直近limit件を返す。
/// FreeDvLogAdapter::recent()と揃えた振る舞いにしてあるが、こちらは
/// QsoRecordの履歴ではなくCSVを都度読み直す(ディスク上の"正"のソースが
/// あるため、UDP受信履歴のようにアプリ内メモリで保持する必要がない)。
pub fn recent(csv_path: &str, limit: usize) -> Vec<RxStationRecord> {
    let mut records = find_all_stations(csv_path);
    records.reverse();
    records.truncate(limit);
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_line() {
        let line = "2026-07-25,12:34:56,JA3MBC,700D,14236000,8";
        let rec = parse_csv_line(line).expect("should parse");
        assert_eq!(rec.date, "2026-07-25");
        assert_eq!(rec.time, "12:34:56");
        assert_eq!(rec.callsign, "JA3MBC");
        assert_eq!(rec.mode, "700D");
        assert_eq!(rec.frequency_hz, 14236000);
        assert_eq!(rec.snr_db, 8);
    }

    #[test]
    fn rejects_malformed_line() {
        assert!(parse_csv_line("2026-07-25,12:34:56,JA3MBC,700D,not_a_number,8").is_none());
        assert!(parse_csv_line("2026-07-25,12:34:56").is_none());
        assert!(parse_csv_line("2026-07-25,12:34:56,,700D,14236000,8").is_none());
    }

    #[test]
    fn skips_header_and_blank_lines() {
        let csv = "date,time,callsign,mode,frequency_hz,snr_db\n\
                    2026-07-25,12:34:56,JA3MBC,700D,14236000,8\n\
                    \n\
                    2026-07-25,12:40:00,JA1XYZ,1600,7150000,-2\n";
        let dir = std::env::temp_dir().join(format!("freedv_rx_log_test_{}.csv", std::process::id()));
        std::fs::write(&dir, csv).unwrap();

        let all = find_all_stations(dir.to_str().unwrap());
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].callsign, "JA3MBC");
        assert_eq!(all[1].callsign, "JA1XYZ");

        let recent1 = recent(dir.to_str().unwrap(), 1);
        assert_eq!(recent1.len(), 1);
        assert_eq!(recent1[0].callsign, "JA1XYZ"); // 新しい順

        std::fs::remove_file(&dir).ok();
    }
}
