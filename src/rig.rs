use crate::flrig;

// 将来構想(未実装): flrig以外のバックエンド(Hamlib, CI-V直結, テスト用ダミー等)にも
// 対応できるよう、下記のようなtraitで抽象化する案がある(ChatGPT提案、2026-07-09)。
// 実装は2つ目のバックエンドが実際に必要になってから着手する。
//
// pub trait RigBackend {
//     fn update(&mut self, state: &mut RigState);
// }

/// リグ(無線機)の現在状態。UI層はこの構造体だけを見る。
pub struct RigState {
    pub frequency: String,
    pub mode: String,
    pub ptt: bool,
}

impl Default for RigState {
    fn default() -> Self {
        Self {
            frequency: "INIT".to_string(),
            mode: "INIT".to_string(),
            ptt: false,
        }
    }
}

impl RigState {
    /// 表示用ラベル("RX"/"TX")
    pub fn ptt_label(&self) -> &'static str {
        if self.ptt { "TX" } else { "RX" }
    }

    /// 周波数をMHz単位の数値として取得できれば返す
    pub fn frequency_mhz(&self) -> Option<f64> {
        self.frequency.parse::<f64>().ok().map(|hz| hz / 1_000_000.0)
    }
}

/// flrigへ問い合わせ、成功した項目だけ state を更新する。
/// (通信失敗時は直前の値を保持する: 既存ui.rsの挙動を踏襲)
pub fn update(state: &mut RigState) {
    if let Ok(freq) = flrig::get_vfo() {
        state.frequency = freq;
    }

    if let Ok(mode) = flrig::get_mode() {
        state.mode = mode;
    }

    if let Ok(ptt) = flrig::get_ptt() {
        state.ptt = ptt;
    }
}
