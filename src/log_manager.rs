use crate::log_adapter::{LogAdapter, QsoRecord};
use crate::wsjtx_log::WsjtxLogAdapter;
use crate::fldigi_log::FldigiLogAdapter;
use crate::freedv_log::FreeDvLogAdapter;
use crate::wsjtx_protocol::QsoLogged;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
    freedv: FreeDvLogAdapter,
}

impl LogManager {
    pub fn new(wsjtx_all_txt_path: String, my_call: String) -> Self {
        let wsjtx = WsjtxLogAdapter::new(
            wsjtx_all_txt_path,
            my_call,
        );

        let fldigi = FldigiLogAdapter::new(
            "~/.fldigi/logbook.adif".to_string(),
        );

        Self {
            adapters: vec![
                Box::new(wsjtx),
                Box::new(fldigi),
            ],
            freedv: FreeDvLogAdapter::new(),
        }
    }

    pub fn latest_qso(&self) -> Option<QsoRecord> {
        for adapter in &self.adapters {
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
            // TODO: ここでADIF/CSV/HAMLOGへの実際の書き込みを行う
        }
    }
}
