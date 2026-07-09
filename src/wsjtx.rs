use crate::rig::RigState;

/// RigState を WSJT-X 向けのログ文字列に変換する。
/// (今はフォーマット変換のみ。実際のUDP送信は未実装 - 次回以降の課題)
pub fn format_for_wsjtx(state: &RigState) -> String {
    let freq_hz = state.frequency.clone();
    format!("FREQ_HZ={} MODE={} PTT={}", freq_hz, state.mode, state.ptt_label())
}
