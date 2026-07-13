mod ui;
mod flrig;
mod config;
mod rig;
mod hamlog;
mod wsjtx_log;
mod fldigi_log;
mod log_adapter;
mod log_manager;

fn main() -> eframe::Result<()> {
    ui::run()
}
