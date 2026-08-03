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

/// 年月日時分秒を、指定した時間(delta_hours)だけシフトする(日付・月・年またぎに対応)。
/// タイムゾーン変換(JST⇔UTCの±9時間)用の共通ユーティリティ。
/// delta_hoursは±24時間未満のシフトを想定(それ以上の大きなシフトは非対応)。
pub fn shift_datetime_hours(
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    delta_hours: i32,
) -> (u32, u32, u32, u32, u32, u32) {
    let mut total_hour = hour as i32 + delta_hours;
    let mut y = year;
    let mut mo = month;
    let mut d = day;

    while total_hour < 0 {
        total_hour += 24;
        if d > 1 {
            d -= 1;
        } else if mo > 1 {
            mo -= 1;
            d = days_in_month(y, mo);
        } else {
            mo = 12;
            y -= 1;
            d = days_in_month(y, mo);
        }
    }
    while total_hour >= 24 {
        total_hour -= 24;
        let dim = days_in_month(y, mo);
        if d < dim {
            d += 1;
        } else if mo < 12 {
            mo += 1;
            d = 1;
        } else {
            mo = 1;
            y += 1;
            d = 1;
        }
    }

    (y, mo, d, total_hour as u32, minute, second)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
