# 協業者向け ブリーフィング(最新状況・AI/人間共通)

このプロジェクトに新しく参加する方(AI・人間を問わず)は、まずこのファイルを読んでください。

## リポジトリ構成(2026-07-09時点)

| 記号 | リポジトリ | 役割 |
|---|---|---|
| H  | ham_control      | 運用版(Stable) — Claudeと継続改善中 |
| H0 | ham_control_0.1  | 記念版(Milestone) — v0.1で完全固定、変更禁止 |
| V  | ham_control_v02  | 開発版(Development) — ChatGPT・Claude・マタノさんで共同開発中 |
| D  | ham_dev_notes    | 開発記録・日誌(番号順のセッションログ) |

## 現在の担当分担

- ham_control (H): マタノさん + Claude
- ham_control_v02 (V): マタノさん + ChatGPT + Claude + その他賛同者
- ham_control_0.1 (H0): 変更しない(誰も触らない)

## 直近の技術状況(v02)

- flrigとXML-RPC通信し、周波数・モード・PTT状態をリアルタイム表示する機能まで実装済み
- N5010のGPU(Intel HD Graphics ILK)はハードウェアレンダリング非対応、`LIBGL_ALWAYS_SOFTWARE=1`が必須
- 日本語フォント未対応のため、UI表示は英語(RX/TX等)で統一している

## 運用ルール

- 「記憶より記録」— GitHubを正本とする
- 過去の記録に番号の欠番・割り込みはしない(常に次の番号を採番)
- 詳しい経緯は各リポジトリの docs/ を参照

## 現在の開発状況

| 項目 | 状態 | 担当 |
|---|---|---|
| rig.rs | 完了(抽象化レイヤーとして実装済み) | ChatGPT・Claude・マタノ |
| config.rs | 完了(flrig接続先・ポーリング間隔を環境変数で上書き可能) | Claude・マタノ |
| hamlog.rs | 完了(TX→RX変化を検知しUDP送信。送信先はHAM_HAMLOG_ADDRで設定可能) | Claude・マタノ |
| wsjtx.rs | 完了(TX→RX変化を検知しUDP送信。送信先はHAM_WSJTX_ADDRで設定可能) | Claude・マタノ |

## 次回作業予定

1. hamlog.rs / wsjtx.rs で送っているUDPメッセージのフォーマットが、
   実際のHAMLOG/WSJT-X側で正しく解釈されるか検証する(現状は独自フォーマットで
   実機側の受信確認はまだ行っていない)
2. 検証結果に応じてフォーマットを標準的なものに合わせる
3. RigBackend trait化(ChatGPT提案、2026-07-09)の着手を検討する
   (2つ目のバックエンドが必要になった時点、という前提条件を満たすか要判断)
