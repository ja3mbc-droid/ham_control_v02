use std::sync::Mutex;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

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
            time_on: qso.date_time_on.clone(),
            time_off: qso.date_time_off.clone(),
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
