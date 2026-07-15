use crate::log_adapter::{LogAdapter, QsoRecord};

pub struct FreeDvLogAdapter;


impl FreeDvLogAdapter {

    pub fn new() -> Self {
        println!("[FreeDV] adapter ready");
        Self
    }




    pub fn from_qso(
        &self,
        qso: &crate::wsjtx_protocol::QsoLogged,
    ) -> Option<QsoRecord> {

        Some(QsoRecord {
            peer_call: qso.dx_call.clone(),
            status: None,
            rst_sent: qso.report_sent.clone(),
            rst_rcvd: qso.report_received.clone(),
            freq_mhz: qso.tx_frequency.to_string(),
            qso_mode: qso.mode.clone(),
            time_on: qso.date_time_on.clone(),
            time_off: qso.date_time_off.clone(),
        })
    }

    pub fn parse(&self, data: &[u8]) -> Option<QsoRecord> {

        println!(
            "[FreeDV] received {} bytes",
            data.len()
        );

        None
    }
}


impl LogAdapter for FreeDvLogAdapter {

    fn latest_qso(&self) -> Option<QsoRecord> {
        None
    }


    fn name(&self) -> &'static str {
        "FreeDV"
    }
}
