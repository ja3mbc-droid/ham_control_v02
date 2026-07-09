use crate::rig::RigState;
use std::net::UdpSocket;

/// RigState を HAMLOG 向けのログ文字列に変換する。
pub fn format_for_hamlog(state: &RigState) -> String {
    let freq_mhz = state
        .frequency_mhz()
        .map(|mhz| format!("{:.3}", mhz))
        .unwrap_or_else(|| "----".to_string());

    format!("FREQ={} MODE={} PTT={}", freq_mhz, state.mode, state.ptt_label())
}

/// HAMLOGへUDPでログ文字列を送信する。
/// 失敗しても呼び出し元(GUI)を止めないよう、エラーはStringで返すだけにする。
pub fn send(state: &RigState, addr: &str) -> Result<(), String> {
    let message = format_for_hamlog(state);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .send_to(message.as_bytes(), addr)
        .map_err(|e| e.to_string())?;

    Ok(())
}
