use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

use crate::wsjtx_protocol::{
    parse_message,
    parse_qso_logged,
    MessageType,
};
use crate::log_manager::LogManager;


pub fn start(log_manager: Arc<LogManager>) {

    thread::spawn(move || {

        let socket = UdpSocket::bind("127.0.0.1:2237")
            .expect("WSJT-X UDP bind failed");

        println!("WSJT-X UDP receiver started :2237");


        loop {

            let mut buf = [0u8; 2048];

            match socket.recv_from(&mut buf) {

                Ok((size, addr)) => {

                    println!(
                        "WSJT-X UDP {} bytes from {}",
                        size,
                        addr
                    );


                    match parse_message(&buf[..size]) {

                        Ok(msg) => {

                            println!(
                                "WSJT message type = {:?}",
                                msg.msg_type
                            );

                            if msg.msg_type == MessageType::QsoLogged {

                                match parse_qso_logged(&msg.payload) {

                                    Ok(qso) => {

                                        println!("QSO Logged {:?}", qso);

                                        log_manager.handle_freedv_qso(&qso);
                                    }

                                    Err(e) => {
                                        println!("QSO parse error: {}", e);
                                    }
                                }
                            }
                        }

                        Err(e) => {
                            println!("Message parse error: {}", e);
                        }
                    }
                }


                Err(e) => {
                    eprintln!("UDP error {}", e);
                }
            }
        }
    });
}
