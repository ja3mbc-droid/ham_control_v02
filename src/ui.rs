use eframe::egui;
use crate::flrig;

pub fn run() -> eframe::Result<()> {
    eframe::run_native(
        "HAM CONTROL v02",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(App)),
    )
}

struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("HAM CONTROL v02");
            ui.separator();
            ui.label("IC-7000");
            ui.label("flrig : READY");
            ui.label("Version : 0.2");

            match flrig::get_vfo() {
                Ok(xml) => {
                    if let Some(p1) = xml.find("<value>") {
                        if let Some(p2) = xml.find("</value>") {
                            let hz = &xml[p1 + 7..p2];
                            if let Ok(freq) = hz.parse::<u64>() {
                                ui.separator();
                                ui.label(format!("Frequency : {:.6} MHz", freq as f64 / 1_000_000.0));
                            }
                        }
                    }
                }
                Err(e) => {
                    ui.separator();
                    ui.label(format!("flrig ERROR : {}", e));
                }
            }
        });
    }
}
