use crate::log_adapter::{LogAdapter, QsoRecord};
use crate::wsjtx_log::WsjtxLogAdapter;
use crate::fldigi_log::FldigiLogAdapter;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
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
        }
    }

    pub fn latest_qso(&self) -> Option<QsoRecord> {
        for adapter in &self.adapters {
            if let Some(qso) = adapter.latest_qso() {
                return Some(qso);
            }
        }

        None
    }
}
