mod ui;
mod flrig;
mod config;
mod rig;
mod hamlog;
mod wsjtx_log;
mod fldigi_log;
mod log_adapter;

fn main() -> eframe::Result<()> {
    ui::run()
}
