# 002_2026-07-13_FD_Linker_FreeDV_HAMLOG調査

## 目的

FreeDV 2.3.1 → FD_Linker Listen Edition Ver2.2.1 → Turbo HAMLOG/Win Ver5.47c の自動転送確認。

2026-07-13 に実施した動作確認、原因調査、未解決点を記録する。


## 環境

- OS: Ubuntu 24.04
- Wine環境使用
- FreeDV 2.3.1
- FD_Linker Listen Edition Ver2.2.1
- Turbo HAMLOG/Win Ver5.47c
- Callsign: JA3MBC
- Grid: PM74QR


## 確認済み事項

### 1. FD_Linker Listen 起動確認

FD_Linker Listen Edition Ver2.2.1 の起動を確認。

表示内容：

FD_Linker Listen[Listen Edition] Ver2.2.1

MY CALL : JA3MBC
MY GL   : PM74QR
PORT    : 2237

起動後、FreeDVからのUDP待受状態になっていることを確認。

## 確認済み事項

### 1. FD_Linker Listen 起動確認

FD_Linker Listen Edition Ver2.2.1 の起動を確認。

表示内容：

FD_Linker Listen[Listen Edition] Ver2.2.1

MY CALL : JA3MBC
MY GL   : PM74QR
PORT    : 2237

起動後、FreeDVからのUDP待受状態になっていることを確認。

## 2026-07-14 追加確認

### HAM局コントロール（実用版）5番 FreeDV起動確認

起動順序を変更した。

変更後:

HAM局コントロール
→ HAMLOG起動
→ FD_Linker起動
→ FreeDV起動

確認結果:

- flrig 起動 OK
- Turbo HAMLOG/Win Ver5.47c 起動 OK
- FD_Linker Listen Edition Ver2.2.1 起動 OK
- FreeDV 2.3.1 起動 OK

### FreeDV UDP確認

tcpdump により UDP port 2237 の通信を確認。

確認データ:

- FreeDV
- ZL2IT
- JA3MBC
- PM74QR
- DIGITALVOICE

結果:

FreeDV → FD_Linker 間のUDP通信は正常。

### HAMLOG転送確認

FreeDVの Log QSO を実行。

結果:

- HAMLOG手動入力は正常
- FD_Linkerは起動・受信可能
- FD_LinkerからHAMLOGへのキー入力転送は未解決

### 今後の方針

FD_LinkerのWine上でのキー入力方式を深追いせず、
ham_control_v02側でFreeDV情報を直接取得する方式を検討する。

候補:

FreeDV UDP 2237
↓
ham_control_v02
↓
内部ログ形式
↓
HAMLOG連携

