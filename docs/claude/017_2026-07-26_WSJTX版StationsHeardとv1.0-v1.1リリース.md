# 017: WSJT-X版Stations Heardの実装とv1.0/v1.1リリース (2026-07-26)

016で実装したFreeDV版Stations Heardに続き、WSJT-X版を追加し、
あわせてVer1.0の完成条件をすべて満たしてGitHub Releaseを公開した。

## ①: ChatGPTによる設計レビュー

WSJT-X版Stations Heardの叩き台設計をChatGPTにレビューしてもらい、
以下の5点で合意した。

- 手動更新のみ(常時監視・毎フレーム読み込みはしない、YAGNI)
- 重複排除なし(FreeDV版と統一。「QSOログ」ではなく「受信履歴」なので、
  同じ局が何度も聞こえた事実も含めて記録する)
- タブを増やすのではなく、Stations Heardパネル内でラジオボタンにより
  WSJT-X/FreeDV/fldigi/MMSSTVを切り替える
- 各ソースが共通の`RxStationRecord`を返す「入力アダプタ」方式を
  採用し、既存の`QsoRecord`入力アダプタ方式と設計思想を揃える
- コードスタイル: `crate::freedv_rx_log::RxStationRecord`の都度
  フルパス記述より`use`文でまとめる

## ②: fldigi/MMSSTVの受信専用ログ有無を実機確認

fldigi/MMSSTVに「聞こえたが交信していない局」を記録する専用ログが
存在するか、`inotifywait -m -r`で受信中のファイル書き込みを実機観察
して確認した。結果、fldigiで書き込みが発生したのは`logbook.adif`
(QSO成立分のみ)と`ui_stats.txt`/`debug/*`(内部統計)のみで、
Rxデコード結果を逐次記録する専用ログは存在しないことが判明した。
予想通りの結果だったため、027ではWSJT-X/FreeDVの2ソースのみを
対象とし、fldigi/MMSSTVは対応データなしとして見送ることにした。

## ③: `wsjtx_log::recent_heard()`の実装

ALL.TXTの全Rx行(CQ行を含む)を対象に、聞こえた局のコールサインを
抽出する関数を追加した。FT8/FT4の標準メッセージは
「`<相手局> <自局> <レポート等>`」、CQは「`CQ <自局> <グリッド>`」
という並びのため、fields[8]が常に「そのdecode行を実際に送信した
(=聞こえた)局」のコールサインになる。これを利用し、
`read_latest_qso()`/`extract_all_qsos()`(sender/receiver判定で
相手局を特定するQSO抽出ロジック)とは完全に分離した、単純な
抽出方式にした。重複排除は行わず、新しい順でlimit件を返す。

テスト作成時、FT8のメッセージ書式(「相手局が先、自局が後」)を
逆に書いたテストデータのバグを一度出したが、`recent_heard()`側の
「fields[8]==my_callの行は除外する」という防御ロジックが正しく
働いて検出できた。実装は正常で、テストデータの方が誤りだった。

## ④: ソース切替UIと入力アダプタ方式への統一

`ui.rs`に`HeardSource`(WsjtX/FreeDv)enumを追加し、Stations Heard
パネル内にラジオボタンで配置した。`refresh_stations_heard()`を
選択中のソースで分岐させ、WSJT-X/FreeDVどちらも戻り値を
`RxStationRecord`に統一。GUI側の表示ロジックはソースを意識しない
構成にした(ChatGPTレビュー④の入力アダプタ方式)。ALL.TXT解析には
自局コールが必要なため、`LogManager`に`my_call()`のgetterを
新設して窓口を用意した。

cargo test 8件全パス、cargo buildエラー0件を確認した後、実機で
WSJT-X/FreeDVそれぞれのラジオボタン選択時に、実際の受信データ
(WSJT-X: ALL.TXTのFT8デコード結果、FreeDV: freedv_rx_log.csv)が
正しく切り替わって表示されることをスクリーンショットで確認した。

## ⑤: v1.0/v1.1のGitHub Release公開

Ver1.0の完成条件リストの最後に残っていた「GitHub Release」を
公開した。git tag `v1.0`(commit `2921608`、023以前の状態)には
既存のリリースノート下書きが残っていたため、それをそのまま採用。
027(WSJT-X版Stations Heard)は`v1.0`タグより後の変更のため、
`v1.1`として別タグ・別リリースにした。

- v1.0: https://github.com/ja3mbc-droid/ham_control_v02/releases/tag/v1.0
  (WSJT-X/FreeDV/fldigi/MMSSTVの4ソース対応、HAMLOG WM_COPYDATA連携などの基準版)
- v1.1: https://github.com/ja3mbc-droid/ham_control_v02/releases/tag/v1.1
  (commit `6e68791`。Stations HeardにWSJT-Xソースを追加)

なお027の実装(`log_manager.rs`/`ui.rs`/`wsjtx_log.rs`)は、一連の
作業の中で一時commit/push漏れになっていたが、v1.1タグ作成前に
気づいて`6e68791`としてcommit・pushし解消した。
