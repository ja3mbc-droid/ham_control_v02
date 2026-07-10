use crate::flrig;
use crate::config::Config;

// 将来構想(未実装): flrig以外のバックエンド(Hamlib, CI-V直結, テスト用ダミー等)にも
// 対応できるよう、下記のようなtraitで抽象化する案がある(ChatGPT提案、2026-07-09)。
// 実装は2つ目のバックエンドが実際に必要になってから着手する。
//
// pub trait RigBackend {
//     fn update(&mut self, state: &mut RigState);
// }

pub struct RigState {
    pub frequency: String,
    pub mode: String,
    pub ptt: bool,
    pub smeter: String,
    pub swr: String,
    pub power: String,
    pub split: bool,
    pub vfo_ab: String,
}

impl Default for RigState {
    fn default() -> Self {
        Self {
            frequency: "INIT".to_string(),
            mode: "INIT".to_string(),
            ptt: false,
            smeter: "INIT".to_string(),
            swr: "INIT".to_string(),
            power: "INIT".to_string(),
            split: false,
            vfo_ab: "INIT".to_string(),
        }
    }
}

impl RigState {
    pub fn ptt_label(&self) -> &'static str {
        if self.ptt { "TX" } else { "RX" }
    }

    pub fn frequency_mhz(&self) -> Option<f64> {
        self.frequency.parse::<f64>().ok().map(|hz| hz / 1_000_000.0)
    }
}

pub fn update(state: &mut RigState, cfg: &Config) {
    if let Ok(freq) = flrig::get_vfo(&cfg.flrig_addr) {
        state.frequency = freq;
    }
    if let Ok(mode) = flrig::get_mode(&cfg.flrig_addr) {
        state.mode = mode;
    }
    if let Ok(ptt) = flrig::get_ptt(&cfg.flrig_addr) {
        state.ptt = ptt;
    }

    if let Ok(smeter) = flrig::get_smeter(&cfg.flrig_addr) {
        state.smeter = smeter;
    }

    if let Ok(swr) = flrig::get_swr(&cfg.flrig_addr) {
        state.swr = swr;
    }

    if let Ok(power) = flrig::get_power(&cfg.flrig_addr) {
        state.power = power;
    }

    if let Ok(split) = flrig::get_split(&cfg.flrig_addr) {
        state.split = split;
    }

    if let Ok(vfo_ab) = flrig::get_vfo_ab(&cfg.flrig_addr) {
        state.vfo_ab = vfo_ab;
    }
}
