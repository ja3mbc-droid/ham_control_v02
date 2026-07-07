use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::flrig;

pub fn run() -> eframe::Result<()> {
    let state = Arc::new(Mutex::new(String::from("INIT")));

    eframe::run_native(
        "HAM CONTROL v02",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(App {
            state,
            last: Instant::now(),
        })),
    )
}

struct App {
    state: Arc<Mutex<String>>,
    last: std::time::Instant,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint();

        // 1秒ごとに通信（UIスレッドで安全）
        if self.last.elapsed() >= Duration::from_secs(1) {
            self.last = Instant::now();

            if let Ok(xml) = flrig::get_vfo() {
                let hz = xml
                    .split("<value>")
                    .nth(1)
                    .and_then(|s| s.split("</value>").next())
                    .unwrap_or("0")
                    .trim()
                    .to_string();

                if let Ok(mut s) = self.state.lock() {
                    *s = hz;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HAM CONTROL v02");
            ui.colored_label(egui::Color32::GREEN, "VISIBLE TEST OK");

            if let Ok(s) = self.state.lock() {
                ui.label(format!("RAW VALUE: {}", *s));

                if let Ok(freq) = s.parse::<f64>() {
                    ui.label(format!("Frequency: {:.6} MHz", freq / 1_000_000.0));
                } else {
                    ui.label("WAITING...");
                }
            }
        });
    }
}
