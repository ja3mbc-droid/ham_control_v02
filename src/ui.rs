use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::flrig;

#[derive(Clone)]
struct RigState {
    freq: String,
    mode: String,
    ptt: String,
}

impl Default for RigState {
    fn default() -> Self {
        Self {
            freq: "INIT".to_string(),
            mode: "INIT".to_string(),
            ptt: "INIT".to_string(),
        }
    }
}

pub fn run() -> eframe::Result<()> {
    let state = Arc::new(Mutex::new(RigState::default()));

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "HAM CONTROL v02",
        options,
        Box::new(|_| Box::new(App {
            state,
            last: Instant::now(),
        })),
    )
}

struct App {
    state: Arc<Mutex<RigState>>,
    last: std::time::Instant,
}

fn extract_value(xml: &str) -> String {
    let inner = xml
        .split("<value>")
        .nth(1)
        .and_then(|s| s.split("</value>").next())
        .unwrap_or("0")
        .trim()
        .to_string();

    // <i4>123</i4> や <boolean>1</boolean> のような内側タグを剥がす
    if inner.starts_with('<') {
        if let Some(gt_pos) = inner.find('>') {
            let after_open = &inner[gt_pos + 1..];
            if let Some(lt_pos) = after_open.find('<') {
                return after_open[..lt_pos].trim().to_string();
            }
        }
    }

    inner
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint();

        // 1秒ごとに通信（UIスレッドで安全）
        if self.last.elapsed() >= Duration::from_secs(1) {
            self.last = Instant::now();

            if let Ok(xml) = flrig::get_vfo() {
                let hz = extract_value(&xml);
                if let Ok(mut s) = self.state.lock() {
                    s.freq = hz;
                }
            }

            if let Ok(xml) = flrig::get_mode() {
                let mode = extract_value(&xml);
                if let Ok(mut s) = self.state.lock() {
                    s.mode = mode;
                }
            }

            if let Ok(xml) = flrig::get_ptt() {
                let ptt_raw = extract_value(&xml);
                let ptt_label = match ptt_raw.as_str() {
                    "1" => "TX".to_string(),
                    "0" => "RX".to_string(),
                    other => format!("UNKNOWN({})", other),
                };
                if let Ok(mut s) = self.state.lock() {
                    s.ptt = ptt_label;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HAM CONTROL v02");
            ui.colored_label(egui::Color32::GREEN, "VISIBLE TEST OK");

            if let Ok(s) = self.state.lock() {
                ui.label(format!("RAW VALUE: {}", s.freq));

                if let Ok(freq) = s.freq.parse::<f64>() {
                    ui.label(format!("Frequency: {:.6} MHz", freq / 1_000_000.0));
                } else {
                    ui.label("WAITING...");
                }

                ui.separator();
                ui.label(format!("MODE: {}", s.mode));

                let color = if s.ptt == "TX" {
                    egui::Color32::RED
                } else {
                    egui::Color32::LIGHT_GREEN
                };
                ui.colored_label(color, format!("STATUS: {}", s.ptt));
            }
        });
    }
}
