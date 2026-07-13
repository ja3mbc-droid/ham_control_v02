use crate::log_adapter::{LogAdapter, QsoRecord};
use crate::wsjtx_log::WsjtxLogAdapter;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
}

impl LogManager {
    pub fn new(wsjtx_all_txt_path: String, my_call: String) -> Self {
        let wsjtx = WsjtxLogAdapter::new(
            wsjtx_all_txt_path,
            my_call,
        );

        Self {
            adapters: vec![
                Box::new(wsjtx),
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
