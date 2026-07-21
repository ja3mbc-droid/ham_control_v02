# 014: WM_COPYDATA方式によるHAMLOG自動入力の実証成功 (2026-07-20)

007/008でxdotoolによるGUI自動化を断念した後、MMSSTV対応(012/013)を経て、
HAMLOGが公式に持つ外部連携機能(WM_COPYDATA)を使った自動入力の再挑戦を行い、
**実機で完全動作を確認した**。

## きっかけ

Turbo HAMLOG/Winには昔からDDE/外部入力/コマンドライン/DLLといった外部連携
機能があり、その資料(TH527API.zip: HAMLOG50.txt/H, Readme.txt, HamlogMs.txt)
を入手した。

## 仕様の要点(Readme.txtより)

HAMLOGはWM_COPYDATAというWindowsメッセージ経由で外部と連携できる。

- `FindWindow("TThwin", NULL)` でHAMLOG本体のウィンドウハンドルを取得
- `SendMessage(hwnd, WM_COPYDATA, 自分のhwnd, &COPYDATASTRUCT)` でコマンド送信
- `COPYDATASTRUCT.dwData` にコマンド番号を指定。主なコマンド:
  - `1〜14`: 個別項目への送信(1=コールサイン, 2=日付, ... 12=QTH, 13=Remarks1, 14=Remarks2)
  - `15`: コールサイン〜Remarks2とチェックボックスをまとめて16行のテキストとして送信
  - `16`: 入力バッファ(LOG-[A]画面)のクリア
  - `18`: [Save]ボタンのクリック相当。`THW_SAVEBOX_OFF`(0x80000)とorすれば
    確認ダイアログ無しで即保存
- 文字列はShift-JISのヌル終端バイト列で渡す

## 壁: Win32 APIはLinuxネイティブから直接呼べない

`FindWindowA`/`SendMessageA`はWindows APIなので、Linuxネイティブの
ham_control_v02からは直接呼べない。そこで、Wine上で動く小さな橋渡し
プログラム`hamlog_bridge`を別途Rustで実装し、`x86_64-pc-windows-gnu`
ターゲットでクロスコンパイルする方針にした。

セットアップ:
```
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

## hamlog_bridgeの実装

標準入力からUTF-8テキストを受け取り、Shift-JISに変換してWM_COPYDATAで
送信するだけのシンプルなコンソールプログラム。

```
echo -n "<テキスト>" | wine hamlog_bridge.exe <dwData(10進数、フラグはor済み)>
```

依存クレート: `windows-sys`(Win32 API FFI)、`encoding_rs`(Shift-JIS変換)

実装時に見つけたバグ: `windows-sys`の`HWND`は生ポインタではなく`isize`型
なので、`.is_null()`ではなく`== 0`で判定する必要がある。

## 実機テストの結果

1. **dwData=1(コールサイン単体送信)**: LOG-[A]画面のCall欄に正しく反映。成功。
2. **dwData=15(16行一括送信)**: 最初、送った内容と入った欄がズレた。
   `L01〜L16`という一意な値を送るテストで、実際の並びを確定させた。
   - 1行目・16行目: チェックボックス関連(空でよい)
   - 2〜15行目: Call, Date, Time, His, My, Freq, Mode, Code, G.L, QSL,
     HisName, QTH, Remarks1, Remarks2 の順
3. **重要な仕様確認**: 空行を送った項目は、既存の内容がクリアされず
   前回の値が残ったままになる(仕様書の「空改行の欄は転送されない」は
   文字通りの意味だった)。そのため、新しいQSOを送る前に必ず
   `dwData=16`(クリア)を先に実行する必要がある。
4. **dwData=16→15の2段階**: クリア後に送信することで、前回のテスト
   残骸が混ざらずクリーンに反映されることを確認。
5. **dwData=18|THW_SAVEBOX_OFF(即保存)**: 実機で保存を実行し、HAMLOG
   本体の交信一覧に実際にレコード(No.1590 ZZTEST3)として登録されて
   いることを確認。

## 結論

`dwData=16(クリア)→dwData=15(全項目送信)→dwData=18|THW_SAVEBOX_OFF(即保存)`
の3コマンドを`hamlog_bridge`経由で送るだけで、GUI操作を一切介さずに
HAMLOGへQSOを完全自動登録できることが実証された。xdotoolのような
座標・ウィンドウの不安定さに悩まされていた007/008とは対照的に、
公式の連携機能を使うこの方式は安定して動作している。

## 気になった点(未解決)

未知のコールサイン(ZZTEST3)を保存した際、QTHが自動で「Brazil」、
Codeが「693A」になる謎の挙動があった。HAMLOGが未知のコールサインに
対して何らかのフォールバック処理(国別コード判定等)を行っている
可能性があるが、原因は未確認。実際の運用コールサインでは起きない
可能性が高いが、次回以降で気に留めておきたい。

## 重要インシデント: テスト残骸が実運用ログに混入

テストで送ったQTH「テスト局3」をクリアしないまま作業を離れたところ、
その後のSSTVロールコールで実際にJA3HWXを手入力保存した際、QTH欄が
空欄のまま保存されたため、テストの残骸がJA3HWXの本物のQSOレコード
(No.1588)に紛れ込んでしまった。

原因は前述の「空行は既存値を上書きしない」仕様そのもの。テストで
一度でも値をセットしたら、たとえテストが終わっても、次に誰かが
(手入力であっても)その項目を空欄のまま保存すると、テストの残骸が
一緒に保存されてしまう。

**教訓: hamlog_bridgeでテストを行った後は、作業を離れる前に必ず
`dwData=16`でクリアすること。** 特に実運用中(ロールコール等)と
並行してテストする場合は要注意。

## 次回やること

- `hamlog_bridge`をham_control_v02本体に統合する(現在は単体の
  スタンドアロンプログラムとしてのみ動作確認済み)
- テスト用のZZTEST3レコードをHAMLOG側から手動削除する
- テスト残骸が紛れ込んだJA3HWX(No.1588)のQTH欄を正しい値に修正する
- 一覧の「済にする」ボタンの動作を、xdotool自動入力の代わりに
  hamlog_bridge経由の自動登録に置き換えることを検討する
