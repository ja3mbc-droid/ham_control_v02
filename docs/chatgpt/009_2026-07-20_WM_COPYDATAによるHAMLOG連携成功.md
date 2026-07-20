# 009_2026-07-20_WM_COPYDATAによるHAMLOG連携成功

## 概要

2026-07-20、Turbo HAMLOGとのWM_COPYDATA連携について実機確認を実施。

従来のGUI自動操作(xdotool等)ではなく、Windowsメッセージ
WM_COPYDATAを利用した直接連携方式の動作確認に成功した。

## 成功した処理

以下の順番でHAMLOG登録が可能であることを確認。

1. dwData=16

入力欄クリア

2. dwData=15

QSO情報全項目送信

3. dwData=18 + THW_SAVEBOX_OFF

即保存

## 実機確認結果

テストコール

ZZTEST3

について、Turbo HAMLOG交信履歴へ登録成功。

No.1590として保存されていることを確認。

SendMessage結果:

SendMessage result: 66076

この戻り値はエラーではなく、メッセージ処理成功を示すものとして確認。

## 意義

WM_COPYDATA方式により、

アプリケーション
    ↓
WM_COPYDATA
    ↓
Turbo HAMLOG

という直接制御経路が成立した。

これにより、不安定な画面操作自動化に依存せず、
確実なHAMLOG自動登録方式を構築できる可能性が高まった。

## ham_control_v02との統合方針

入力側:

- WSJT-X
- FreeDV
- fldigi
- MMSSTV

を共通QSO形式へ変換。

その後、

QsoRecord
    ↓
HAMLOG Bridge
    ↓
WM_COPYDATA
    ↓
Turbo HAMLOG

という構成を目指す。

## MMSSTVについて

今回確認したWM_COPYDATA方式は、MMSSTV連携にも重要。

MMSSTV専用アダプタ(mmsstv_log.rs)を追加することで、
他のログソフトと同じ流れでHAMLOG登録できる可能性がある。

## 次回課題

- Rust版HAMLOG Bridge整理
- QSOデータ構造からWM_COPYDATA送信
- dwData=18保存処理の正式組込み
- MMSSTVデータ取得方式検討

