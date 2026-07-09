use crate::rig::RigState;
use std::net::UdpSocket;

/// RigState を WSJT-X 向けのログ文字列に変換する。
pub fn format_for_wsjtx(state: &RigState) -> String {
    let freq_hz = state.frequency.clone();
    format!("FREQ_HZ={} MODE={} PTT={}", freq_hz, state.mode, state.ptt_label())
}

/// WSJT-Xへ(仮)UDPでログ文字列を送信する。
pub fn send(state: &RigState, addr: &str) -> Result<(), String> {
    let message = format_for_wsjtx(state);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .send_to(message.as_bytes(), addr)
        .map_err(|e| e.to_string())?;

    Ok(())
}
