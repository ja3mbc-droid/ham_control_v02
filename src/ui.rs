use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::rig::{self, RigState};
use crate::config::{self, Config};
use crate::hamlog;
use crate::wsjtx;

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
            hamlog_status: String::new(),
            wsjtx_status: String::new(),
        })),
    )
}

struct App {
    state: Arc<Mutex<RigState>>,
    last: std::time::Instant,
    cfg: Config,
    prev_ptt: bool,
    hamlog_status: String,
    wsjtx_status: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint();

        if self.last.elapsed() >= Duration::from_millis(self.cfg.poll_interval_ms) {
            self.last = Instant::now();
            if let Ok(mut s) = self.state.lock() {
                rig::update(&mut s, &self.cfg);

                // TX -> RX の変化を検知したらHAMLOGへ送信(送信終了=QSOの区切りとみなす)
                if self.prev_ptt && !s.ptt {
                    match hamlog::send(&s, &self.cfg.hamlog_addr) {
                        Ok(_) => self.hamlog_status = "HAMLOG: SENT OK".to_string(),
                        Err(e) => self.hamlog_status = format!("HAMLOG: SEND FAILED ({})", e),
                    }

                    match wsjtx::send(&s, &self.cfg.wsjtx_addr) {
                        Ok(_) => self.wsjtx_status = "WSJTX: SENT OK".to_string(),
                        Err(e) => self.wsjtx_status = format!("WSJTX: SEND FAILED ({})", e),
                    }
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
            }

            if !self.hamlog_status.is_empty() {
                ui.separator();
                ui.label(&self.hamlog_status);
            }

            if !self.wsjtx_status.is_empty() {
                ui.label(&self.wsjtx_status);
            }
        });
    }
}
