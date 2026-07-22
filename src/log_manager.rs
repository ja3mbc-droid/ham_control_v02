use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};
use crate::wsjtx_log::WsjtxLogAdapter;
use crate::fldigi_log::FldigiLogAdapter;
use crate::freedv_log::FreeDvLogAdapter;
use crate::mmsstv_log::MmsstvLogAdapter;
use crate::wsjtx_protocol::QsoLogged;
use std::collections::HashSet;
use std::sync::Mutex;

pub struct LogManager {
    adapters: Vec<Box<dyn LogAdapter>>,
    freedv: FreeDvLogAdapter,
    activity_log_path: String,
    adif_path: String,
    wsjtx_all_txt_path: String,
    fldigi_logbook_path: String,
    mmsstv_mdt_path: String,
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
        adif_path: String,
        fldigi_logbook_path: String,
        mmsstv_mdt_path: String,
    ) -> Self {
        let wsjtx = WsjtxLogAdapter::new(
            wsjtx_all_txt_path.clone(),
            my_call.clone(),
        );

        let fldigi = FldigiLogAdapter::new(
            fldigi_logbook_path.clone(),
        );

        let mmsstv = MmsstvLogAdapter::new(
            mmsstv_mdt_path.clone(),
        );

        // 起動時に既存の活動ログCSVを読み込み、既に記録済みのQSOキーを
        // 事前に把握しておく(過去分の再書き込みを防ぐ)。
        let written_wsjtx_keys = Mutex::new(load_existing_keys(&activity_log_path));

        Self {
            adapters: vec![
                Box::new(wsjtx),
                Box::new(fldigi),
                Box::new(mmsstv),
            ],
            freedv: FreeDvLogAdapter::new(),
            activity_log_path,
            adif_path,
            wsjtx_all_txt_path,
            fldigi_logbook_path,
            mmsstv_mdt_path,
            my_call,
            written_wsjtx_keys,
        }
    }

    /// ALL.TXTを再走査し、まだ書き込んでいないQSOがあれば、その場でCSV/ADIFへ
    /// 即時追記する。GUIでの表示/未表示に関わらずデータを残すためのバックグラウンド処理。
    /// wsjtx_poller(数秒おきのタイマースレッド)から呼ばれる想定。
    ///
    /// Complete(73/RR73確認済み)だけでなく、Incomplete(尻切れ・応答が途中で止まった)
    /// QSOも記録する。これは「後日その局から交信の有無を問い合わせられた際の裏付け」
    /// として残しておきたい、というユーザーの意向による。COMMENT1に[73未確認]と
    /// 付記することで、GUIやCSVを見ればひと目でComplete/Incompleteの区別がつく。
    /// NoResponse(RSTレポートすら交換されていない)は、単発の空振り受信ノイズを
    /// 記録で埋めないよう、対象外とする。
    pub fn catch_up_wsjtx(&self) {
        let records = crate::wsjtx_log::extract_all_qsos(&self.wsjtx_all_txt_path, &self.my_call);

        for record in records {
            let comment1 = match record.status {
                Some(QsoStatus::Complete) => "",
                Some(QsoStatus::Incomplete) => "[73未確認]",
                Some(QsoStatus::NoResponse) | None => continue,
            };

            // 状態(Complete/Incomplete)ごとにキーを分けることで、
            // 「最初は尻切れとして記録→後で73が来て完了」となった場合に
            // 尻切れの記録を消さず、完了の記録を別途追加できるようにする。
            let key = format!("{}|{}|{:?}", record.peer_call, record.time_on, record.status);

            {
                let mut seen = self.written_wsjtx_keys.lock().unwrap();
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
            }

            println!("[LogManager] catch-up: new WSJT-X QSO ({:?}) {:?}", record.status, record);

            match crate::hamlog::append_log_from_record(&record, comment1, &self.activity_log_path) {
                Ok(()) => {
                    println!("[LogManager] wrote WSJT-X QSO to {}", self.activity_log_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write WSJT-X QSO: {}", e);
                }
            }

            match crate::hamlog::append_adif_from_record(&record, comment1, &self.adif_path) {
                Ok(()) => {
                    println!("[LogManager] wrote WSJT-X QSO to {}", self.adif_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write WSJT-X QSO to ADIF: {}", e);
                }
            }
        }
    }

    /// GUIの「直近の交信一覧」表示用。ALL.TXTを走査し、直近limit件のQSOを
    /// 新しい順(最新が先頭)で返す。NoResponse(空振り)は一覧に出さない。
    /// latest_qso_by_source()と違い1件だけでなく複数件返すため、パイルアップ時に
    /// 間の局が見えなくなる問題をGUI側からも確認できるようにする。
    pub fn recent_wsjtx_qsos(&self, limit: usize) -> Vec<QsoRecord> {
        let mut records = crate::wsjtx_log::extract_all_qsos(&self.wsjtx_all_txt_path, &self.my_call);

        records.retain(|r| r.status != Some(QsoStatus::NoResponse) && r.status.is_some());

        // extract_all_qsos()はファイル中で各セッションが開始した順(古い→新しい)で返すため、
        // 新しい順に並べ替えてから直近limit件を取る
        records.reverse();
        records.truncate(limit);
        records
    }

    /// GUIの「直近の交信一覧」表示用(fldigi版)。recent_wsjtx_qsos()と同じ形。
    /// logbook.adifは記録順(古い→新しい)で並んでいるため、同様に反転してから
    /// 直近limit件を返す。
    pub fn recent_fldigi_qsos(&self, limit: usize) -> Vec<QsoRecord> {
        let mut records = crate::fldigi_log::find_all_qsos(&self.fldigi_logbook_path);
        records.reverse();
        records.truncate(limit);
        records
    }

    /// GUIの「直近の交信一覧」表示用(MMSSTV版)。.MDTは記録順(古い→新しい)で
    /// 並んでいるため、fldigi版と同様に反転してから直近limit件を返す。
    pub fn recent_mmsstv_qsos(&self, limit: usize) -> Vec<QsoRecord> {
        let mut records = crate::mmsstv_log::find_all_qsos(&self.mmsstv_mdt_path);
        records.reverse();
        records.truncate(limit);
        records
    }

    /// GUIの「直近の交信一覧」表示用(FreeDV版)。FreeDvLogAdapter側で
    /// 既に新しい順・limit件に整形されているため、そのまま委譲する。
    pub fn recent_freedv_qsos(&self, limit: usize) -> Vec<QsoRecord> {
        self.freedv.recent(limit)
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

            match crate::hamlog::append_log_from_record(&record, "", &self.activity_log_path) {
                Ok(()) => {
                    println!("[LogManager] wrote FreeDV QSO to {}", self.activity_log_path);
                }
                Err(e) => {
                    eprintln!("[LogManager] failed to write FreeDV QSO: {}", e);
                }
            }

            match crate::hamlog::append_adif_from_record(&record, "", &self.adif_path) {
                Ok(()) => {
                    println!("[LogManager] wrote FreeDV QSO to {}", self.adif_path);
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

/// 既存の活動ログCSVを読み込み、"CALL|TIME_ON|Status"形式のキー集合を作る。
/// CSV列順は hamlog::csv_header() のとおり:
/// TIME_ON,TIME_OFF,FREQ,MODE,CALL,RST_SENT,RST_RCVD,COMMENT1,COMMENT2
/// COMMENT1に[73未確認]が入っていればIncomplete、それ以外はCompleteとして扱う
/// (catch_up_wsjtx()のキー生成ロジックと一致させる必要がある)。
fn load_existing_keys(path: &str) -> HashSet<String> {
    let mut keys = HashSet::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return keys,
    };

    for line in content.lines() {
        if line.starts_with("TIME_ON,") {
            // ヘッダー行(019で追加)はデータとして扱わない
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 5 {
            continue;
        }
        let time_on = fields[0];
        let call = fields[4];
        let comment1 = fields.get(7).copied().unwrap_or("");

        let status = if comment1.contains("73未確認") {
            Some(QsoStatus::Incomplete)
        } else {
            Some(QsoStatus::Complete)
        };

        keys.insert(format!("{}|{}|{:?}", call, time_on, status));
    }

    keys
}
