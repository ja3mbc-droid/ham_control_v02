# 007_2026-07-15 HAM CONTROL v02 GUI起動確認

## 確認結果

HAM CONTROL v02 の起動確認。

正常確認：

- Ubuntu サイドメニュー起動
  - GUI表示 OK

- HAM局コントロール13番起動
  - GUI表示 OK

- release版起動

LIBGL_ALWAYS_SOFTWARE=1 ~/ham_control_v02/target/release/ham_control_v02

  - GUI表示 OK

## 結論

HAM CONTROL v02 の通常起動は正常。

残課題：
cargo run 時のみGUI文字表示なし。

