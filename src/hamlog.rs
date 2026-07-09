use crate::rig::RigState;

/// RigState を HAMLOG 向けのログ文字列に変換する。
/// (今はフォーマット変換のみ。実際のUDP送信は未実装 - 次回以降の課題)
pub fn format_for_hamlog(state: &RigState) -> String {
    let freq_mhz = state
        .frequency_mhz()
        .map(|mhz| format!("{:.3}", mhz))
        .unwrap_or_else(|| "----".to_string());

    format!("FREQ={} MODE={} PTT={}", freq_mhz, state.mode, state.ptt_label())
}
