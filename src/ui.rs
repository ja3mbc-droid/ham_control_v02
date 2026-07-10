use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::rig::{self, RigState};
use crate::config::{self, Config};
use crate::hamlog;

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
            cfg: config::load(),
            prev_ptt: false,
            tx_started_at: None,
            tx_started_unix: 0,
            log_status: String::new(),
        })),
    )
}

struct App {
    state: Arc<Mutex<RigState>>,
    last: std::time::Instant,
    cfg: Config,
    prev_ptt: bool,
    tx_started_at: Option<Instant>,
    tx_started_unix: u64,
    log_status: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint();

        if self.last.elapsed() >= Duration::from_millis(self.cfg.poll_interval_ms) {
            self.last = Instant::now();
            if let Ok(mut s) = self.state.lock() {
                rig::update(&mut s, &self.cfg);

                // RX -> TX: 送信開始時刻を記録
                if !self.prev_ptt && s.ptt {
                    self.tx_started_at = Some(Instant::now());
                    self.tx_started_unix = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                }

                // TX -> RX: 送信時間を計算してログに記録
                if self.prev_ptt && !s.ptt {
                    let tx_seconds = self
                        .tx_started_at
                        .map(|t| t.elapsed().as_secs_f64())
                        .unwrap_or(0.0);

                    match hamlog::append_log(&s, &self.cfg.activity_log_path, self.tx_started_unix) {
                        Ok(_) => self.log_status = format!("LOG: SAVED (TX {:.1}s)", tx_seconds),
                        Err(e) => self.log_status = format!("LOG: FAILED ({})", e),
                    }
                    self.tx_started_at = None;
                }

                self.prev_ptt = s.ptt;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HAM CONTROL v02");
            ui.colored_label(egui::Color32::GREEN, "VISIBLE TEST OK");

            if let Ok(s) = self.state.lock() {
                ui.label(format!("RAW VALUE: {}", s.frequency));

                if let Some(mhz) = s.frequency_mhz() {
                    ui.label(format!("Frequency: {:.6} MHz", mhz));
                } else {
                    ui.label("WAITING...");
                }

                ui.separator();
                ui.label(format!("MODE: {}", s.mode));

                let color = if s.ptt {
                    egui::Color32::RED
                } else {
                    egui::Color32::LIGHT_GREEN
                };
                ui.colored_label(color, format!("STATUS: {}", s.ptt_label()));

                ui.separator();
                ui.label(format!("S-METER: {}", s.smeter));
                ui.label(format!("SWR: {}", s.swr));
                ui.label(format!("POWER: {}", s.power));
                ui.label(format!("SPLIT: {}", if s.split { "ON" } else { "OFF" }));
                ui.label(format!("VFO: {}", s.vfo_ab));

                ui.separator();
                ui.label("--- DEBUG: SWR RAW XML ---");
                ui.label(&s.swr_raw_xml);
            }

            if !self.log_status.is_empty() {
                ui.separator();
                ui.label(&self.log_status);
            }
        });
    }
}
