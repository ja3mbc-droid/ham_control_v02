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
mod wsjtx_receiver;
mod wsjtx_poller;
mod hamlog_auto;

use std::sync::Arc;
use log_manager::LogManager;

fn main() -> eframe::Result<()> {
    let cfg = config::load();

    let log_manager = Arc::new(LogManager::new(
        cfg.wsjtx_all_txt_path.clone(),
        "JA3MBC".to_string(),
        cfg.activity_log_path.clone(),
        cfg.fldigi_logbook_path.clone(),
    ));

    wsjtx_receiver::start(log_manager.clone());
    wsjtx_poller::start(log_manager.clone(), 5);
    ui::run(log_manager.clone())
}
