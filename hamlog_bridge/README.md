# hamlog_bridge

HAMLOG(Turbo HAMLOG/Win)へWM_COPYDATAでコマンドを送るための、Wine上で動かす
小さな橋渡しプログラム。詳細は docs/claude/014 を参照。

## ビルド (要 mingw-w64, rustup target x86_64-pc-windows-gnu)
```
cargo build --target x86_64-pc-windows-gnu
```

## 使い方
```
echo -n "<UTF-8テキスト>" | wine target/x86_64-pc-windows-gnu/debug/hamlog_bridge.exe <dwData>
```

現状はham_control_v02本体には未統合。単体プログラムとして動作確認済み。
