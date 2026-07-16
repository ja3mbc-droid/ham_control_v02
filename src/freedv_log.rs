use std::sync::Mutex;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

pub struct FreeDvLogAdapter {
    last_qso: Mutex<Option<QsoRecord>>,
}


impl FreeDvLogAdapter {

    pub fn new() -> Self {
        println!("[FreeDV] adapter ready");
        Self {
            last_qso: Mutex::new(None),
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

    /// UDP受信でQSOが確定するたびにLogManagerから呼ばれ、
    /// 最新の1件として保持する。GUI等からのpollはlatest_qso()経由で
    /// この値を読む。
    pub fn store_latest(&self, record: QsoRecord) {
        if let Ok(mut guard) = self.last_qso.lock() {
            *guard = Some(record);
        }
    }
}


impl LogAdapter for FreeDvLogAdapter {

    fn latest_qso(&self) -> Option<QsoRecord> {
        self.last_qso.lock().ok()?.clone()
    }


    fn name(&self) -> &'static str {
        "FreeDV"
    }
}
