use std::fs;
use crate::log_adapter::{LogAdapter, QsoRecord, QsoStatus};

/// MMSSTVの.MDTログファイル(バイナリ)を読むAdapter。
///
/// フォーマットは012(docs/claude)の調査結果に基づく。MMSSTVはLGPLで
/// 公開されている(n5ac/mmsstv on GitHub)ため、LogFile.hのSDMMLOG構造体
/// 定義を元にオフセットを特定し、実データとの照合で確認済み。
///
/// ただし band/mode/fq(周波数・モード)のバイト値→実際の文字列への
/// 変換テーブルはソースから未特定のため、今回は生の数値をそのまま
/// 文字列化する形にとどめている(WSJT-X/fldigiほど正確な表示にはならない)。
pub struct MmsstvLogAdapter {
    pub mdt_path: String,
}

impl MmsstvLogAdapter {
    pub fn new(mdt_path: String) -> Self {
        Self { mdt_path }
    }
}

impl LogAdapter for MmsstvLogAdapter {
    fn latest_qso(&self) -> Option<QsoRecord> {
        find_all_qsos(&self.mdt_path).into_iter().last()
    }

    fn name(&self) -> &'static str {
        "MMSSTV"
    }
}

/// ヘッダー部のオフセット(この後からレコードが並ぶ)
const FHDOFF: usize = 256;
/// 1レコードのサイズ = SDMMLOG構造体(256byte) + 検索用索引領域(16byte)
const RECORD_SIZE: usize = 256 + 16;

// SDMMLOG構造体内の各フィールドのオフセット・サイズ(パディング無し前提)
const OFF_YEAR: usize = 0; // BYTE
const OFF_DATE: usize = 1; // WORD
const OFF_BTIME: usize = 3; // WORD
const OFF_ETIME: usize = 5; // WORD
const OFF_CALL: usize = 7;
const LEN_CALL: usize = 17; // MLCALL+1
const OFF_UR: usize = 24;
const LEN_UR: usize = 21; // MLRST+1 (送ったレポート)
const OFF_MY: usize = 45;
const LEN_MY: usize = 21; // MLRST+1 (受けたレポート)
const OFF_REM: usize = 181;
const LEN_REM: usize = 57; // MLREM+1 (記事)

/// null終端されたバイト列をShift-JISとしてデコードし、末尾の空白/NUL/制御文字を落とす。
/// MMSSTVはASCII/Shift-JISが混在した固定長フィールドをNUL埋めで持っている。
fn decode_field(bytes: &[u8]) -> String {
    // NUL(0x00)以降は未使用領域なので切り捨てる
    let trimmed = match bytes.iter().position(|&b| b == 0) {
        Some(pos) => &bytes[..pos],
        None => bytes,
    };
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(trimmed);
    let s = if had_errors {
        // 変換に失敗した場合はASCIIとして最善努力でデコード
        String::from_utf8_lossy(trimmed).to_string()
    } else {
        decoded.to_string()
    };
    s.trim().to_string()
}

/// date(WORD)・btime/etime(WORD)を、DOS FAT形式(推定)としてデコードする。
/// date: 上位バイトが月寄り、下位バイトが日寄りのパック値(未確定、暫定実装)
/// time: 5bit時 + 6bit分 + 5bit秒/2 (FATタイムスタンプ形式)
/// 012での検証で妥当な値になることを確認しているが、別レコードでの
/// 追加検証はまだ済んでいないため、変換結果はあくまで暫定表示として扱う。
fn decode_datetime(year_byte: u8, date: u16, time: u16) -> String {
    let year = if year_byte < 50 {
        2000 + year_byte as u32
    } else {
        1900 + year_byte as u32
    };
    let month = (date >> 5) & 0x0f;
    let day = date & 0x1f;
    let hour = (time >> 11) & 0x1f;
    let minute = (time >> 5) & 0x3f;
    let second = (time & 0x1f) * 2;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

/// .MDTファイルから全QSOを、ファイル中の記録順(古い→新しい)で読み取る。
/// find_all_qsos()という関数名・シグネチャはwsjtx_log.rs/fldigi_log.rsと
/// 揃えてあり、log_manager.rs側から同じパターンで呼べるようにしている。
pub fn find_all_qsos(mdt_path: &str) -> Vec<QsoRecord> {
    let data = match fs::read(mdt_path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    if data.len() <= FHDOFF {
        // ヘッダーのみ、またはそれ未満 = レコード0件
        return Vec::new();
    }

    let record_area = &data[FHDOFF..];
    let record_count = record_area.len() / RECORD_SIZE;

    let mut records = Vec::with_capacity(record_count);
    for i in 0..record_count {
        let base = i * RECORD_SIZE;
        let rec = &record_area[base..base + RECORD_SIZE];

        let call = decode_field(&rec[OFF_CALL..OFF_CALL + LEN_CALL]);
        if call.is_empty() {
            // コールサインが空のレコード(削除済み等)はスキップ
            continue;
        }

        let year_byte = rec[OFF_YEAR];
        let date = u16::from_le_bytes([rec[OFF_DATE], rec[OFF_DATE + 1]]);
        let btime = u16::from_le_bytes([rec[OFF_BTIME], rec[OFF_BTIME + 1]]);
        let etime = u16::from_le_bytes([rec[OFF_ETIME], rec[OFF_ETIME + 1]]);

        let ur = decode_field(&rec[OFF_UR..OFF_UR + LEN_UR]);
        let my = decode_field(&rec[OFF_MY..OFF_MY + LEN_MY]);
        let rem = decode_field(&rec[OFF_REM..OFF_REM + LEN_REM]);

        records.push(QsoRecord {
            peer_call: call,
            // MMSSTVはログに保存された時点で交信完結とみなし、
            // WSJT-Xのような73確認判定は行わずCompleteとする
            status: Some(QsoStatus::Complete),
            rst_sent: ur,
            rst_rcvd: my,
            // band/mode/fqの変換テーブルが未確定のため、周波数欄には
            // 記事欄(rem)の内容を暫定的に流用する(GL等が書かれていることが多い)
            freq_mhz: rem,
            qso_mode: "MMSSTV".to_string(),
            time_on: decode_datetime(year_byte, date, btime),
            time_off: decode_datetime(year_byte, date, etime),
        });
    }

    records
}
