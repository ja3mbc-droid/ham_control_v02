# 012_2026-07-17_WSJT-X_session_fix作業記録

## 本日の作業目的

WSJT-X ALL.TXT読込処理のsession分離修正。

## 実施内容

- 旧extract_all_qsos()は相手コールサイン単位で結合していた。
- 同一局との別日・別時間交信を分離するためsession方式へ変更。
- SESSION_GAP_SECS=900を導入。
- parse_ymd_hms()追加。
- time_str所有権エラー修正。

## ビルド結果

cargo build --release 成功。

## 実機確認

正常表示:

[FreeDV] adapter ready
WSJT-X ALL.TXT catch-up poller started (interval 5s)

問題:

WSJT-X UDP bind failed:
Address already in use

原因:
UDP受信ポート競合。

## 到達点

完了:
- extract_all_qsos session版反映
- release build成功
- CSV/ADI生成確認済み

未確認:
- session分離後のcatch-up実機確認
- HAMLOG再確認

## 次回開始

1. UDP 2237使用状況確認
2. WSJT-X receiver確認
3. session分離結果確認
