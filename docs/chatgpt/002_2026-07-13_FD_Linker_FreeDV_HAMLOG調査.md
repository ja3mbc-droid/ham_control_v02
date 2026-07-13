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
