use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};
use crate::wsjtx_log::WsjtxLogAdapter;
use crate::fldigi_log::FldigiLogAdapter;
use crate::freedv_log::FreeDvLogAdapter;
use crate::wsjtx_protocol::QsoLogged;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
    freedv: FreeDvLogAdapter,
    activity_log_path: String,
    wsjtx_all_txt_path: String,
    my_call: String,
    /// 既にCSV/ADIFへ書き込み済みのWSJT-X QSOを覚えておくためのキー集合
    /// (peer_call + time_on)。パイルアップ時、ALL.TXTを繰り返し走査しても
    /// 同じ交信を二重に書き込まないようにするための重複排除。
    written_wsjtx_keys: Mutex<HashSet<String>>,
}

impl LogManager {
    pub fn new(
        wsjtx_all_txt_path: String,
        my_call: String,
        activity_log_path: String,
        fldigi_logbook_path: String,
    ) -> Self {
        let wsjtx = WsjtxLogAdapter::new(
            wsjtx_all_txt_path.clone(),
            my_call.clone(),
        );

        let fldigi = FldigiLogAdapter::new(
            fldigi_logbook_path,
        );

        // 起動時に既存の活動ログCSVを読み込み、既に記録済みのQSOキーを
        // 事前に把握しておく(過去分の再書き込みを防ぐ)。
        let written_wsjtx_keys = Mutex::new(load_existing_keys(&activity_log_path));

        Self {
            adapters: vec![
                Box::new(wsjtx),
                Box::new(fldigi),
            ],
            freedv: FreeDvLogAdapter::new(),
            activity_log_path,
            wsjtx_all_txt_path,
            my_call,
            written_wsjtx_keys,
        }
    }

    /// ALL.TXTを再走査し、まだ書き込んでいない「完了済み(73確認済み)」の
    /// QSOがあれば、その場でCSV/ADIFへ即時追記する。
    /// GUIでの表示/未表示に関わらずデータを残すためのバックグラウンド処理。
    /// wsjtx_poller(数秒おきのタイマースレッド)から呼ばれる想定。
    pub fn catch_up_wsjtx(&self) {
        let records = crate::wsjtx_log::extract_all_qsos(&self.wsjtx_all_txt_path, &self.my_call);

        for record in records {
            if record.status != Some(QsoStatus::Complete) {
                // 73/RR73が来ていない(尻切れ・空振り)QSOは、まだ確定していないので書き込まない
                continue;
            }

            let key = format!("{}|{}", record.peer_call, record.time_on);

            {
                let mut seen = self.written_wsjtx_keys.lock().unwrap();
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
            }

            println!("[LogManager] catch-up: new completed WSJT-X QSO {:?}", record);

            match crate::hamlog::append_log_from_record(&record, &self.activity_log_path) {
                Ok(()) => {
                    println!("[LogManager] wrote WSJT-X QSO to {}", self.activity_log_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write WSJT-X QSO: {}", e);
                }
            }

            let adif_path = format!(
                "{}/ham_control_v02.adi",
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
            );
            match crate::hamlog::append_adif_from_record(&record, &adif_path) {
                Ok(()) => {
                    println!("[LogManager] wrote WSJT-X QSO to {}", adif_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write WSJT-X QSO to ADIF: {}", e);
                }
            }
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

    /// GUIのラジオボタン等から、特定のソース名を指定して取得する。
    /// 名前は各AdapterのLogAdapter::name()と一致させる("FreeDV","WSJT-X","fldigi")。
    pub fn latest_qso_by_source(&self, source: &str) -> Option<QsoRecord> {
        if source == "FreeDV" {
            return self.freedv.latest_qso();
        }

        self.adapters
            .iter()
            .find(|a| a.name() == source)
            .and_then(|a| a.latest_qso())
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

/// 既存の活動ログCSVを読み込み、"CALL|TIME_ON"形式のキー集合を作る。
/// CSV列順は hamlog::csv_header() のとおり:
/// TIME_ON,TIME_OFF,FREQ,MODE,CALL,RST_SENT,RST_RCVD,COMMENT1,COMMENT2
fn load_existing_keys(path: &str) -> HashSet<String> {
    let mut keys = HashSet::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return keys,
    };

    for line in content.lines() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 5 {
            continue;
        }
        let time_on = fields[0];
        let call = fields[4];
        keys.insert(format!("{}|{}", call, time_on));
    }

    keys
}
