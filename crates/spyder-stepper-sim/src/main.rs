//! TCP firmware simulator for the spyder stepper protocol.
//!
//! Run: `cargo run -p spyder-stepper-sim -- 9002`
//! Then: `spyder play ... --backend stepper --device 127.0.0.1:9002`

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn handle(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let _ = writeln!(stream, "OK spyder-stepper-sim");
    let mut line = String::new();
    let mut positions = vec![0i64; 8];
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }
        if cmd == "H" {
            for p in &mut positions {
                *p = 0;
            }
            let _ = writeln!(stream, "OK");
            continue;
        }
        if cmd == "P" {
            let mut out = String::from("P");
            for p in &positions {
                out.push_str(&format!(" {p}"));
            }
            let _ = writeln!(stream, "{out}");
            continue;
        }
        if let Some(rest) = cmd.strip_prefix('M') {
            let parts: Vec<_> = rest.split_whitespace().collect();
            if parts.is_empty() {
                let _ = writeln!(stream, "ERR bad");
                continue;
            }
            let n: usize = parts[0].parse().unwrap_or(0);
            if n == 0 || parts.len() < 1 + 2 * n {
                let _ = writeln!(stream, "ERR parse");
                continue;
            }
            for i in 0..n {
                let s: i64 = parts[1 + 2 * i].parse().unwrap_or(0);
                if i < positions.len() {
                    positions[i] += s;
                }
                // delay ignored in sim
            }
            let _ = writeln!(stream, "OK");
            continue;
        }
        let _ = writeln!(stream, "ERR unknown");
    }
}

fn main() {
    let port = env::args()
        .nth(1)
        .unwrap_or_else(|| "9002".into());
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).expect("bind");
    eprintln!("spyder-stepper-sim listening on {addr}");
    for conn in listener.incoming() {
        match conn {
            Ok(stream) => {
                eprintln!("client connected");
                std::thread::spawn(move || handle(stream));
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
}
