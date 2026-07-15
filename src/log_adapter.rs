/// QSOの状態(モジュールを問わず共通)
#[derive(Debug, PartialEq, Clone)]
pub enum QsoStatus {
    Complete,
    Incomplete,
    NoResponse,
}

/// どのログソフト(WSJT-X, fldigi, 将来のFreeDV等)から得られた情報でも
/// 共通の形で扱うためのレコード。008の設計思想(特定ソフトに依存しない)
/// をコードで体現する、中心的なデータ構造。
#[derive(Debug, Clone, Default)]
pub struct QsoRecord {
    pub peer_call: String,
    pub status: Option<QsoStatus>,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub freq_mhz: String,
    pub qso_mode: String,
    pub time_on: String,
    pub time_off: String,
}

/// 各ログソフト用アダプタが実装すべき共通インターフェース。
/// RigBackend(リグ制御の抽象化)と対になる、ログ取得側の抽象化。
pub trait LogAdapter: Send + Sync {
    /// 直近のQSO情報を1件取得する。取得できなければNoneを返す。
    fn latest_qso(&self) -> Option<QsoRecord>;

    /// このアダプタが対応しているソフト名(表示用)
    fn name(&self) -> &'static str;
}
