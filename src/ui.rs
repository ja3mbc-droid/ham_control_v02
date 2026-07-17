use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::rig::{self, RigState};
use crate::config::{self, Config};
use crate::hamlog;
use crate::log_manager::LogManager;
use crate::log_adapter::QsoStatus;

pub fn run(log_manager: Arc<LogManager>) -> eframe::Result<()> {
    let state = Arc::new(Mutex::new(RigState::default()));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 850.0]),
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

            let cfg = config::load();

            Box::new(App {
            state,
            last: Instant::now(),
            cfg,
            log_manager,
            prev_ptt: false,
            tx_started_at: None,
            tx_started_unix: 0,
            log_status: String::new(),
            callsign_input: String::new(),
            comment1_input: String::new(),
            comment2_input: String::new(),
            rst_sent_input: String::new(),
            rst_rcvd_input: String::new(),
            qso_mode: String::new(),
            last_time_on: String::new(),
            last_time_off: String::new(),
            name_input: String::new(),
            qth_input: String::new(),
            gl_input: String::new(),
            code_input: String::new(),
            qsl_via_input: String::new(),
            qsl_sent_input: String::new(),
            qsl_rcvd_input: String::new(),
            log_source_selected: "WSJT-X".to_string(),
            })
        }),
    )
}

struct App {
    state: Arc<Mutex<RigState>>,
    last: std::time::Instant,
    cfg: Config,
    log_manager: Arc<LogManager>,
    prev_ptt: bool,
    tx_started_at: Option<Instant>,
    tx_started_unix: u64,
    log_status: String,
    callsign_input: String,
    comment1_input: String,
    comment2_input: String,
    rst_sent_input: String,
    rst_rcvd_input: String,
    qso_mode: String,
    last_time_on: String,
    last_time_off: String,
    name_input: String,
    qth_input: String,
    gl_input: String,
    code_input: String,
    qsl_via_input: String,
    qsl_sent_input: String,
    qsl_rcvd_input: String,
    log_source_selected: String,
}

impl App {
    /// QsoRecordの内容をCALL/RST等の入力欄へ反映する。
    /// 「選択ソースから読込」ボタンと、直近の交信一覧のどちらからも呼ばれる共通処理。
    fn apply_qso_record(&mut self, source_label: &str, info: crate::log_adapter::QsoRecord) {
        match info.status {
            Some(QsoStatus::Complete) => {
                self.callsign_input = info.peer_call;
                self.rst_sent_input = info.rst_sent;
                self.rst_rcvd_input = info.rst_rcvd;
                self.last_time_on = info.time_on;
                self.last_time_off = info.time_off;
                self.qso_mode = info.qso_mode.clone();
                self.comment1_input.clear();
                self.log_status = format!("{}: 完全成立のQSOを読込みました ({} MHz, {})", source_label, info.freq_mhz, info.qso_mode);
            }
            Some(QsoStatus::Incomplete) => {
                self.callsign_input = info.peer_call;
                self.rst_sent_input = info.rst_sent;
                self.rst_rcvd_input = info.rst_rcvd;
                self.last_time_on = info.time_on;
                self.last_time_off = info.time_off;
                self.qso_mode = info.qso_mode.clone();
                self.comment1_input = "[73未確認]".to_string();
                self.log_status = format!("{}: 尻切れQSOを読込みました(73未確認)", source_label);
            }
            Some(QsoStatus::NoResponse) => {
                self.log_status = format!("{}: 空振り(応答なし)のため読込みません", source_label);
            }
            None => {
                self.log_status = format!("{}: QSO状態情報なし", source_label);
            }
        }
    }
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

            if !self.qso_mode.is_empty() {
                ui.label(format!("QSO MODE: {}", self.qso_mode));
            }
            ui.horizontal(|ui| {
                ui.label("ログソース:");
                ui.radio_value(&mut self.log_source_selected, "WSJT-X".to_string(), "WSJT-X");
                ui.radio_value(&mut self.log_source_selected, "FreeDV".to_string(), "FreeDV");
                ui.radio_value(&mut self.log_source_selected, "fldigi".to_string(), "fldigi");
            });
            if ui.button("選択ソースから読込").clicked() {
                if let Some(info) = self.log_manager.latest_qso_by_source(&self.log_source_selected) {
                    let source = self.log_source_selected.clone();
                    self.apply_qso_record(&source, info);
                } else {
                    self.log_status = format!("{}: 該当データが見つかりません", self.log_source_selected);
                }
            }

            if self.log_source_selected == "WSJT-X" {
                ui.separator();
                ui.label("直近の交信一覧(WSJT-X, クリックで読込):");
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let recent = self.log_manager.recent_wsjtx_qsos(10);
                        if recent.is_empty() {
                            ui.label("(まだ交信データがありません)");
                        }
                        for record in recent {
                            let status_label = match record.status {
                                Some(QsoStatus::Complete) => "完了",
                                Some(QsoStatus::Incomplete) => "尻切れ",
                                _ => "?",
                            };
                            let row_label = format!(
                                "{}  {}  {} MHz  {}  [{}]",
                                record.time_on, record.peer_call, record.freq_mhz, record.qso_mode, status_label
                            );
                            if ui.button(row_label).clicked() {
                                self.apply_qso_record("WSJT-X", record);
                            }
                        }
                    });
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
