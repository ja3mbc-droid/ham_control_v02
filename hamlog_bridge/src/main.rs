// hamlog_bridge: HAMLOG(Turbo HAMLOG/Win)へWM_COPYDATAでコマンドを送る、
// Wine上で動かすための小さな橋渡しプログラム。
//
// ham_control_v02(Linuxネイティブ)は、FindWindowA/SendMessageAといった
// Win32 APIを直接呼べないため、`wine hamlog_bridge.exe <dwData> < payload`
// という形で子プロセスとしてこれを起動し、実際のWM_COPYDATA送信だけを
// 代行させる。
//
// 使い方:
//   echo -n "<送るテキスト(UTF-8, 空でもよい)>" | hamlog_bridge.exe <dwData(10進数)>
//
// 例: dwData=15(全項目送信)なら、16行分のテキストを標準入力から渡す。
//     dwData=18|0x80000(即保存)なら、標準入力は空でよい。
//     フラグ(THW_ENTER等)は呼び出し側であらかじめdwDataにorしておくこと。

use std::io::Read;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::DataExchange::COPYDATASTRUCT;
use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowA, SendMessageA, WM_COPYDATA};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: hamlog_bridge <dwData(10進数、フラグはorした値)>");
        std::process::exit(2);
    }

    let dw_data: usize = match args[1].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("dwDataは10進数の整数で指定してください: {}", args[1]);
            std::process::exit(2);
        }
    };

    // 標準入力からペイロード文字列(UTF-8)を読み、HAMLOGが要求する
    // Shift-JISのヌル終端バイト列に変換する。
    let mut payload_utf8 = String::new();
    std::io::stdin()
        .read_to_string(&mut payload_utf8)
        .expect("標準入力の読み込みに失敗しました");

    let (sjis_bytes, _, had_errors) = encoding_rs::SHIFT_JIS.encode(&payload_utf8);
    if had_errors {
        eprintln!("警告: Shift-JISへの変換で一部の文字を正しく変換できませんでした");
    }
    // ヌル終端を付ける(cbDataはヌル文字分を含めた長さ、仕様書の記述通り)
    let mut buf: Vec<u8> = sjis_bytes.into_owned();
    buf.push(0);

    // "TThwin" もヌル終端したASCIIバイト列で渡す(FindWindowAはANSI版)
    let class_name = b"TThwin\0";

    unsafe {
        let hwnd: HWND = FindWindowA(class_name.as_ptr(), std::ptr::null());
        if hwnd == 0 {
            eprintln!("HAMLOGのウィンドウ(TThwin)が見つかりません。起動していますか?");
            std::process::exit(1);
        }

        let cds = COPYDATASTRUCT {
            dwData: dw_data,
            cbData: buf.len() as u32,
            lpData: buf.as_mut_ptr() as *mut core::ffi::c_void,
        };

        // 第三引数(送信元ウィンドウハンドル)は本来なら自分自身のウィンドウハンドルを
        // 渡すべきだが、hamlog_bridgeはウィンドウを持たないコンソールプログラムのため
        // 0を渡す。HAMLOG側はこの値をSetForegroundWindow等の対象にするだけなので、
        // 0でもコマンド自体の実行(データ登録)には支障が無いと判断している。
        let result = SendMessageA(hwnd, WM_COPYDATA, 0, &cds as *const COPYDATASTRUCT as isize);

        println!("SendMessage result: {}", result);
    }
}
