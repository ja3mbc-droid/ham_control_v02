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
        // 優先順位付き探索
        // 現在は adapters の登録順を優先順位とする。
        // 将来:
        //   1. WSJT-X
        //   2. fldigi
        //   3. FreeDV
        // などへ容易に拡張できる。
        for adapter in &self.adapters {
            if let Some(qso) = adapter.latest_qso() {
                println!("[LogManager] using {}", adapter.name());
                return Some(qso);
            }
        }

        None
    }
}
