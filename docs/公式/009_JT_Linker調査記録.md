# 009_JT_Linker調査記録

## 経緯

WSJT-X経由でHAMLOGへ転送する際、JT_Linkerが毎回エラーを出していることが
実運用中に判明した。マタノさんいわく、以前から発生していたが実害がない
ため無視してきたとのこと。この機会に原因を調査した。

## エラー内容

JT_Linker.Form1.sendHamlog() 内で
「Object reference not set to an instance of an object」(null参照エラー)。
トレースにFindWindow32が含まれる。

## 調査結果

JT_Linker.ini を確認したところ、以下の設定が見つかった。

- DataFromName=(Hamlog)
- DataFromQTH=(Hamlog)

これは、JT_LinkerがHAMLOG本体のウィンドウを探しに行き、名前・QTH情報を
取得しようとしていることを示す。JT_LinkerはWSJT-X側のデータ(ALL.TXT等)
とHAMLOG側のデータ(ウィンドウ経由での取得)の、双方に依存する仲介役
である。そのため、どちらか一方の状態やタイミングがずれるだけで、
処理全体が不安定になりやすい構造だと考えられる。

Wine環境下でのウィンドウハンドル取得(FindWindow32)が不安定になって
いる可能性が高いと推測されるが、これ以上の追及は行わなかった。

## 結論・方針

- JT_Linker経由のアプローチはこれ以上追及しない(調査完了)
- ソースコード(ham_control_v02)には組み込まない
- 新たな互換対応・機能追加の対象としない
- 「ALL.TXT → ham_control_v02(wsjtx_log.rs) → HAMLOG手動入力」という
  経路を正式に採用する

この結論は、マタノさん・Claude・ChatGPTの3者で合意済み(2026-07-12〜13)。
008_特定ソフトに依存しない設計思想.md とも合致する。

## 今後の運用ソフトウェア面での変更

ham_control(運用版)の 2) WSJT-X起動 ボタンから、JT_Linker起動を外す。
(理由: 使わないと決めたソフトを毎回自動起動する必要がないため)
