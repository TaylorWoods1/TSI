//! TCP firmware simulator for the spyder stepper protocol.

use std::env;
use std::net::TcpListener;

use spyder_stepper_sim::SimState;

fn main() {
    let port = env::args().nth(1).unwrap_or_else(|| "9002".into());
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).expect("bind");
    eprintln!("spyder-stepper-sim listening on {addr}");
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                eprintln!("client connected");
                std::thread::spawn(move || {
                    use std::io::BufReader;
                    let mut sim = SimState::new(8);
                    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
                    let mut writer = stream;
                    sim.serve_connection(&mut reader, &mut writer);
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}
