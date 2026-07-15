mod ui;
mod flrig;
mod config;
mod rig;
mod hamlog;
mod wsjtx_log;
mod wsjtx_protocol;
mod fldigi_log;
mod freedv_log;
mod log_adapter;
mod log_manager;

fn main() -> eframe::Result<()> {
    wsjtx_receiver::start();
    ui::run()
}

mod wsjtx_receiver;
