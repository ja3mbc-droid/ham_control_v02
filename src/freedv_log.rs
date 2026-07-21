use std::sync::Mutex;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

/// "YYYYMMDD_HHMMSS"を"YYYY-MM-DD HH:MM:SS"に正規化する。
/// 形式が想定と違う場合は元の文字列をそのまま返す(壊さない)。
fn normalize_freedv_datetime(s: &str) -> String {
    let parts: Vec<&str> = s.splitn(2, '_').collect();
    if parts.len() != 2 || parts[0].len() != 8 || parts[1].len() < 6 {
        return s.to_string();
    }
    let date = parts[0];
    let time = parts[1];
    format!(
        "{}-{}-{} {}:{}:{}",
        &date[0..4], &date[4..6], &date[6..8],
        &time[0..2], &time[2..4], &time[4..6]
    )
}

pub struct FreeDvLogAdapter {
    /// 受信したQSOを古い順に積んでいく履歴。UDPプッシュはこのアプリの
    /// プロセス内でしか観測できないため、WSJT-X(ALL.TXT)やfldigi(logbook.adif)
    /// と違いディスク上の"正"のソースが無く、アプリ再起動で履歴はリセットされる。
    history: Mutex<Vec<QsoRecord>>,
}


impl FreeDvLogAdapter {

    pub fn new() -> Self {
        println!("[FreeDV] adapter ready");
        Self {
            history: Mutex::new(Vec::new()),
        }
    }

    pub fn from_qso(
        &self,
        qso: &crate::wsjtx_protocol::QsoLogged,
    ) -> Option<QsoRecord> {

        Some(QsoRecord {
            peer_call: qso.dx_call.clone(),
            // FreeDVの「Confirm Log...」でOKされた時点で交信は完結しているため、
            // WSJT-Xのような73確認判定は不要でCompleteとする
            status: Some(QsoStatus::Complete),
            rst_sent: qso.report_sent.clone(),
            rst_rcvd: qso.report_received.clone(),
            freq_mhz: format!("{:.6}", qso.tx_frequency as f64 / 1_000_000.0),
            qso_mode: qso.mode.clone(),
            // "YYYYMMDD_HHMMSS"を、wsjtx_log.rs/fldigi_log.rsと揃えた
            // "YYYY-MM-DD HH:MM:SS"形式に正規化する
            time_on: normalize_freedv_datetime(&qso.date_time_on),
            time_off: normalize_freedv_datetime(&qso.date_time_off),
        })
    }

    /// UDP受信でQSOが確定するたびにLogManagerから呼ばれ、履歴の末尾に積む。
    /// GUI等からのpollはlatest_qso()/recent()経由でこの履歴を読む。
    pub fn store_latest(&self, record: QsoRecord) {
        if let Ok(mut guard) = self.history.lock() {
            guard.push(record);
        }
    }

    /// GUIの「直近の交信一覧」表示用。新しい順(最新が先頭)で直近limit件を返す。
    /// recent_wsjtx_qsos()/find_all_qsos()と揃えた形にすることで、ui.rs側が
    /// ログソースによらず同じ表示ロジックを使い回せるようにする。
    pub fn recent(&self, limit: usize) -> Vec<QsoRecord> {
        let guard = match self.history.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let mut records: Vec<QsoRecord> = guard.clone();
        records.reverse();
        records.truncate(limit);
        records
    }
}


impl LogAdapter for FreeDvLogAdapter {

    fn latest_qso(&self) -> Option<QsoRecord> {
        self.history.lock().ok()?.last().cloned()
    }


    fn name(&self) -> &'static str {
        "FreeDV"
    }
}
