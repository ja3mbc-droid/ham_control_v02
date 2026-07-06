mod ui;
mod flrig;
mod config;
mod rig;
mod wsjtx;
mod hamlog;

fn main() -> eframe::Result<()> {
    println!("{}", flrig::get_vfo().unwrap_or_else(|e| format!("ERROR: {}", e)));
    ui::run()
}
