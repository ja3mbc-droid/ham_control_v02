# 018: FreeDV RADEV1 CSVレポート不具合の調査とupstream Issue報告

## 背景

Stations Heard機能(WSJT-X/FreeDV 2ソース版、027パッチで実装済み)の実機確認中、
FreeDV 2.4.0-dev-a4ae + RADEV1モードにおいて、~/.local/share/freedv/freedv_rx_log.csv
が受信成功時にも更新されない事象を確認。

音声デコード自体は正常に完了しているにもかかわらず、freedv_rx_log.csvへの追記が
発生しないため、Stations Heard機能がFreeDV RADEV1受信局を拾えない状態だった。

## 調査内容

- ham_control_v02側の実装(freedv_rx_log.rs)ではなく、FreeDV本体(freedv-gui)側の
  問題であることをログとソースコード調査で切り分け
- 起動ログではCsvReporterが正しく生成されており、ログファイルパスも正しいことを確認
  (`CsvReporter: opening log file ...`は出力される)
- しかし受信時に出るはずの `Reporting callsign ...` および
  `CsvReporter: adding record ...` のログが一切出力されないことを確認
- freedv-gui のソースを確認したところ、`CsvReporter::addReceiveRecord()` は
  `src/main.cpp` 内の `obj->addReceiveRecord(...)` からのみ呼ばれており、
  これは有効なコールサインが取得できた後にのみ実行される経路であることが判明
- 実機テストでは、RADEV1同期は成功し音声デコードも正常に行われる一方、
  受信コールサインはoverが完全に終わった後もUNKのままであることを確認
  → addReceiveRecord() 自体が呼ばれない、というのが根本原因
- FreeDV 2.3.1では同一環境・同一条件で正常にコールサインが確定し、
  freedv_rx_log.csvも正常に更新されることを確認済み(2.4.0-dev-a4aeのみのリグレッション)

## 対応

ChatGPTと共同でIssue文面(Environment/Steps to Reproduce/Expected/Actual/
Investigation/Comparison/Impact/Question/Logの構成)をレビュー・作成し、
upstream の drowe67/freedv-gui リポジトリへ報告。

- Issue #1446: "RADEV1: freedv_rx_log.csv is no longer updated because
  CsvReporter::addReceiveRecord() is never called"
- URL: https://github.com/drowe67/freedv-gui/issues/1446
- 事前に既存Issueとの重複がないことを検索で確認済み
- 調査にClaude/ChatGPTの支援を受けた旨を本文に明記(透明性のため)

## 影響・今後

- 本件はham_control_v02側のバグではなく、FreeDV本体側のリグレッションであるため、
  ham_control_v02側での対応(ワークアラウンド)は現時点では不要と判断
- upstream側の開発者からの返信(意図的な仕様変更か、リグレッションかの回答)待ち
- 返信内容によっては、Stations Heard機能のFreeDV RADEV1ソースが一時的に
  「コールサイン確定分のみ」しか拾えない制限として、UIやドキュメントに
  注記を追加することも検討
