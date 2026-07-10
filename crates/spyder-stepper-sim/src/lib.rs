//! Stepper firmware protocol handler (testable core).

use std::io::{BufRead, Write};

/// In-memory stepper simulator state.
#[derive(Clone, Debug, Default)]
pub struct SimState {
    /// Cumulative step positions per axis.
    pub positions: Vec<i64>,
}

impl SimState {
    /// Create with `n` axes.
    pub fn new(n: usize) -> Self {
        Self {
            positions: vec![0; n],
        }
    }

    /// Handle one protocol line; returns response line without trailing newline.
    pub fn handle_line(&mut self, cmd: &str) -> String {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return String::new();
        }
        if cmd == "H" {
            for p in &mut self.positions {
                *p = 0;
            }
            return "OK".into();
        }
        if cmd == "E" {
            return "OK estop".into();
        }
        if cmd == "P" {
            let mut out = String::from("P");
            for p in &self.positions {
                out.push_str(&format!(" {p}"));
            }
            return out;
        }
        if let Some(rest) = cmd.strip_prefix('M') {
            let parts: Vec<_> = rest.split_whitespace().collect();
            if parts.is_empty() {
                return "ERR bad".into();
            }
            let n: usize = parts[0].parse().unwrap_or(0);
            if n == 0 || parts.len() < 1 + 2 * n {
                return "ERR parse".into();
            }
            for i in 0..n {
                let s: i64 = parts[1 + 2 * i].parse().unwrap_or(0);
                if i < self.positions.len() {
                    self.positions[i] += s;
                }
            }
            return "OK".into();
        }
        "ERR unknown".into()
    }

    /// Serve one client connection until EOF.
    pub fn serve_connection<R: BufRead, W: Write>(&mut self, reader: &mut R, writer: &mut W) {
        let _ = writeln!(writer, "OK spyder-stepper-sim");
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
            let resp = self.handle_line(&line);
            if resp.is_empty() {
                continue;
            }
            let _ = writeln!(writer, "{resp}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_accumulates_positions() {
        let mut sim = SimState::new(4);
        assert_eq!(sim.handle_line("M 2 100 1000 -50 2000"), "OK");
        assert_eq!(sim.positions[0], 100);
        assert_eq!(sim.positions[1], -50);
    }

    #[test]
    fn position_report_returns_steps() {
        let mut sim = SimState::new(2);
        sim.handle_line("M 2 10 1 20 1");
        assert_eq!(sim.handle_line("P"), "P 10 20");
    }

    #[test]
    fn home_zeros_positions() {
        let mut sim = SimState::new(2);
        sim.handle_line("M 2 5 1 7 1");
        sim.handle_line("H");
        assert_eq!(sim.positions, vec![0, 0]);
    }

    #[test]
    fn unknown_command_errors() {
        let mut sim = SimState::new(1);
        assert_eq!(sim.handle_line("Z"), "ERR unknown");
    }
}
