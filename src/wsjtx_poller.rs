use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::log_manager::LogManager;

/// ALL.TXTを一定間隔で再走査し、まだ書き込んでいない完了済みWSJT-X QSOが
/// あればその場でCSV/ADIFへ即時追記するバックグラウンドスレッドを起動する。
///
/// GUIの「WSJT-Xから読込」ボタン(latest_qso方式、最新1件のみ表示)とは独立して
/// 動くため、パイルアップ等で短時間に複数局と交信成立しても、ボタン操作の
/// タイミングに関わらずデータが失われない。
///
/// interval_secs: 走査間隔(秒)。ALL.TXTはWSJT-XがQSO確定時に追記するだけの
/// 軽いテキストファイルなので、数秒間隔でも負荷は小さい。
pub fn start(log_manager: Arc<LogManager>, interval_secs: u64) {
    thread::spawn(move || {
        println!(
            "WSJT-X ALL.TXT catch-up poller started (interval {}s)",
            interval_secs
        );

        loop {
            log_manager.catch_up_wsjtx();
            thread::sleep(Duration::from_secs(interval_secs));
        }
    });
}
