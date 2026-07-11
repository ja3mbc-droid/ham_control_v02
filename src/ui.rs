use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::rig::{self, RigState};
use crate::config::{self, Config};
use crate::hamlog;
use crate::wsjtx_log::{self, QsoStatus};

pub fn run() -> eframe::Result<()> {
    let state = Arc::new(Mutex::new(RigState::default()));

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "HAM CONTROL v02",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            let font_bytes = include_bytes!("../assets/NotoSansCJK-Regular.ttc");
            fonts.font_data.insert(
                "notosans_cjk".to_owned(),
                egui::FontData::from_static(font_bytes),
            );
            fonts.families.get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "notosans_cjk".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("notosans_cjk".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Box::new(App {
            state,
            last: Instant::now(),
            cfg: config::load(),
            prev_ptt: false,
            tx_started_at: None,
            tx_started_unix: 0,
            log_status: String::new(),
            callsign_input: String::new(),
            comment1_input: String::new(),
            comment2_input: String::new(),
            rst_sent_input: String::new(),
            rst_rcvd_input: String::new(),
            last_time_on: String::new(),
            last_time_off: String::new(),
            name_input: String::new(),
            qth_input: String::new(),
            gl_input: String::new(),
            code_input: String::new(),
            qsl_via_input: String::new(),
            qsl_sent_input: String::new(),
            qsl_rcvd_input: String::new(),
            })
        }),
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
    callsign_input: String,
    comment1_input: String,
    comment2_input: String,
    rst_sent_input: String,
    rst_rcvd_input: String,
    last_time_on: String,
    last_time_off: String,
    name_input: String,
    qth_input: String,
    gl_input: String,
    code_input: String,
    qsl_via_input: String,
    qsl_sent_input: String,
    qsl_rcvd_input: String,
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
                    self.last_time_on = hamlog::format_unix_secs_pub(self.tx_started_unix);
                    self.last_time_off = String::new();
                }

                // TX -> RX: 送信時間を計算してログに記録
                if self.prev_ptt && !s.ptt {
                    let tx_seconds = self
                        .tx_started_at
                        .map(|t| t.elapsed().as_secs_f64())
                        .unwrap_or(0.0);

                    self.last_time_off = hamlog::now_string_pub();

                    match hamlog::append_log(
                        &s,
                        &self.cfg.activity_log_path,
                        self.tx_started_unix,
                        &self.callsign_input,
                        &self.comment1_input,
                        &self.comment2_input,
                        &self.rst_sent_input,
                        &self.rst_rcvd_input,
                    ) {
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
                egui::CollapsingHeader::new("DEBUG")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label("--- SWR RAW XML ---");
                        ui.label(&s.swr_raw_xml);
                    });
            }

            ui.separator();
            ui.label(format!("TIME_ON: {}", self.last_time_on));
            ui.label(format!("TIME_OFF: {}", self.last_time_off));
            ui.separator();

            if ui.button("ALL.TXTから読込").clicked() {
                if let Some((peer, status)) = wsjtx_log::find_latest_qso(&self.cfg.wsjtx_all_txt_path, "JA3MBC") {
                    match status {
                        QsoStatus::Complete => {
                            self.callsign_input = peer;
                            self.log_status = "ALL.TXT: 完全成立のQSOを読込みました".to_string();
                        }
                        QsoStatus::Incomplete => {
                            self.callsign_input = peer;
                            self.comment1_input = "[73未確認]".to_string();
                            self.log_status = "ALL.TXT: 尻切れQSOを読込みました(73未確認)".to_string();
                        }
                        QsoStatus::NoResponse => {
                            self.log_status = "ALL.TXT: 空振り(応答なし)のため読込みません".to_string();
                        }
                    }
                } else {
                    self.log_status = "ALL.TXT: 該当データが見つかりません".to_string();
                }
            }

            ui.columns(2, |cols| {
                cols[0].label("CALL:");
                cols[0].text_edit_singleline(&mut self.callsign_input);
                cols[0].label("NAME:");
                cols[0].text_edit_singleline(&mut self.name_input);
                cols[0].label("QTH:");
                cols[0].text_edit_singleline(&mut self.qth_input);
                cols[0].label("G.L:");
                cols[0].text_edit_singleline(&mut self.gl_input);
                cols[0].label("CODE:");
                cols[0].text_edit_singleline(&mut self.code_input);
                cols[0].label("RST SENT:");
                cols[0].text_edit_singleline(&mut self.rst_sent_input);
                cols[0].label("RST RCVD:");
                cols[0].text_edit_singleline(&mut self.rst_rcvd_input);

                cols[1].label("COMMENT1:");
                cols[1].text_edit_singleline(&mut self.comment1_input);
                cols[1].label("COMMENT2:");
                cols[1].text_edit_singleline(&mut self.comment2_input);
                cols[1].label("QSL VIA:");
                cols[1].text_edit_singleline(&mut self.qsl_via_input);
                cols[1].label("QSL SENT:");
                cols[1].text_edit_singleline(&mut self.qsl_sent_input);
                cols[1].label("QSL RCVD:");
                cols[1].text_edit_singleline(&mut self.qsl_rcvd_input);
            });

            if !self.log_status.is_empty() {
                ui.separator();
                ui.label(&self.log_status);
            }
        });
    }
}
