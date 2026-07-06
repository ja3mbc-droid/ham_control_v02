use eframe::egui;

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
        });
    }
}
