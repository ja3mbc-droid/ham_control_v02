use std::net::UdpSocket;

use crate::log_adapter::{LogAdapter, QsoRecord};

pub struct FreeDvLogAdapter;

impl FreeDvLogAdapter {
    pub fn new() -> Self {
        println!("[FreeDV] opening UDP 2237...");

        match UdpSocket::bind("127.0.0.1:2237") {
            Ok(_) => println!("[FreeDV] UDP bind OK"),
            Err(e) => println!("[FreeDV] UDP bind ERROR: {}", e),
        }

        Self
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
