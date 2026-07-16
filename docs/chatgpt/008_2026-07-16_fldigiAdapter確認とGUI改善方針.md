# 008_2026-07-16 fldigi Adapter確認とGUI改善方針

## 本日の目的

ham_control_v02 の残作業である

- fldigi Adapter

について現状確認と動作検証を行った。

---

## 確認結果

### 1. LogManager

LogManager には

- WSJT-X Adapter
- fldigi Adapter
- FreeDV Adapter

の3種類が組み込まれていることを確認した。

---

### 2. fldigiログ保存先

当初

~/.fldigi/logbook.adif

を使用していたが、

実際の環境では

~/.fldigi/logs/logbook.adif

が正しい保存場所であることを確認した。

config.rs を修正し、

fldigi_logbook_path を正しいパスへ変更した。

---

### 3. デバッグログ追加

fldigi_log.rs

へデバッグ表示を追加し、

latest_qso() の呼び出し確認が出来る状態にした。

---

### 4. 動作確認

GUIから

「ALL.TXTから読込」

を実行すると、

LogManager のログより

WSJT-X

が最初に選択されていることを確認。

そのため

fldigi Adapter

まで処理が到達しないことを確認した。

---

### 5. 設計上の課題

現在は

LogManager

が

FreeDV
↓

WSJT-X
↓

fldigi

の優先順位で検索している。

この構造では

WSJT-X にログが存在する限り

fldigi が選択されない。

---

## 今後の改善方針

GUI に

・WSJT-X

・FreeDV

・fldigi

の選択機能を追加し、

ユーザーが読込元を明示的に選択できる設計へ変更する。

これにより

LogManager の優先順位に依存しない構成とする。

---

## 現在の状況

FreeDV Adapter    : 完成

WSJT-X Adapter    : 完成

LogManager統合    : 完成

ADIF出力          : 完成

HAMLOG連携        : 完成

fldigi Adapter    : パス修正完了

GUI切替機能       : 次回実装予定

---

73 de JA3MBC
