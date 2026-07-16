use crate::log_adapter::{LogAdapter, QsoRecord};
use crate::wsjtx_log::WsjtxLogAdapter;
use crate::fldigi_log::FldigiLogAdapter;
use crate::freedv_log::FreeDvLogAdapter;
use crate::wsjtx_protocol::QsoLogged;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
    freedv: FreeDvLogAdapter,
    activity_log_path: String,
}

impl LogManager {
    pub fn new(
        wsjtx_all_txt_path: String,
        my_call: String,
        activity_log_path: String,
        fldigi_logbook_path: String,
    ) -> Self {
        let wsjtx = WsjtxLogAdapter::new(
            wsjtx_all_txt_path,
            my_call,
        );

        let fldigi = FldigiLogAdapter::new(
            fldigi_logbook_path,
        );

        Self {
            adapters: vec![
                Box::new(wsjtx),
                Box::new(fldigi),
            ],
            freedv: FreeDvLogAdapter::new(),
            activity_log_path,
        }
    }

    pub fn latest_qso(&self) -> Option<QsoRecord> {
        println!("[LogManager] latest_qso() called");

        // FreeDVはリアルタイムUDPプッシュのため最優先で確認する
        println!("[LogManager] checking FreeDV");
        if let Some(qso) = self.freedv.latest_qso() {
            println!("[LogManager] using FreeDV");
            return Some(qso);
        }

        println!("[LogManager] checking adapters");

        for adapter in &self.adapters {
            println!("[LogManager] trying {}", adapter.name());

            if let Some(qso) = adapter.latest_qso() {
                println!("[LogManager] using {}", adapter.name());
                return Some(qso);
            }
        }

        None
    }

    /// FreeDVのUDP受信(wsjtx_receiver)から呼ばれる。
    /// 唯一のFreeDvLogAdapter所有者としてQsoRecordへの変換と保存を担う。
    pub fn handle_freedv_qso(&self, qso: &QsoLogged) {
        if let Some(record) = self.freedv.from_qso(qso) {
            println!("[LogManager] FreeDV QSORecord {:?}", record);

            match crate::hamlog::append_log_from_record(&record, &self.activity_log_path) {
                Ok(()) => {
                    println!("[LogManager] wrote FreeDV QSO to {}", self.activity_log_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write FreeDV QSO: {}", e);
                }
            }

            let adif_path = format!(
                "{}/ham_control_v02.adi",
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
            );
            match crate::hamlog::append_adif_from_record(&record, &adif_path) {
                Ok(()) => {
                    println!("[LogManager] wrote FreeDV QSO to {}", adif_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write FreeDV QSO to ADIF: {}", e);
                }
            }

            // GUI等のlatest_qso()pollから見えるよう最新値として保持
            self.freedv.store_latest(record);
        }
    }
}
