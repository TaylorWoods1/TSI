//! Multi-axis stepper backend over the spyder line protocol.
//!
//! Host → firmware:
//! ```text
//! M <n> <s0> <d0_us> <s1> <d1_us> ...\n
//! ```
//! Firmware replies `OK\n` (or `ERR ...\n`).

use crate::transport::Transport;
use crate::{MotorBackend, Result, RuntimeError};

/// Stepper backend speaking the spyder multi-axis protocol.
pub struct StepperBackend {
    transport: Box<dyn Transport>,
    steps: Vec<i64>,
    expect_ack: bool,
}

impl StepperBackend {
    /// Wrap a transport for `n` axes.
    pub fn new(transport: Box<dyn Transport>, n: usize) -> Self {
        Self {
            transport,
            steps: vec![0; n],
            expect_ack: true,
        }
    }

    /// Disable waiting for `OK`.
    pub fn without_ack(mut self) -> Self {
        self.expect_ack = false;
        self
    }

    fn send_line(&mut self, line: &str) -> Result<()> {
        let mut msg = line.to_string();
        if !msg.ends_with('\n') {
            msg.push('\n');
        }
        self.transport.write_all(msg.as_bytes())
    }

    fn expect_ok(&mut self) -> Result<()> {
        if !self.expect_ack {
            return Ok(());
        }
        let resp = self.transport.read_line()?;
        if resp.starts_with("OK") {
            Ok(())
        } else {
            Err(RuntimeError::Backend(format!(
                "stepper firmware: {resp}"
            )))
        }
    }

    /// Ask firmware to zero its counters.
    pub fn home(&mut self) -> Result<()> {
        self.send_line("H")?;
        self.expect_ok()?;
        for s in &mut self.steps {
            *s = 0;
        }
        Ok(())
    }
}

impl MotorBackend for StepperBackend {
    fn axis_count(&self) -> usize {
        self.steps.len()
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<()> {
        if steps.len() != self.steps.len() || delays_s.len() != self.steps.len() {
            return Err(RuntimeError::Config(
                "step/delay length mismatch".into(),
            ));
        }
        let n = steps.len();
        let mut line = format!("M {n}");
        for i in 0..n {
            let delay_us = (delays_s[i] * 1_000_000.0).round().clamp(0.0, 60_000_000.0) as u64;
            let delay_us = if steps[i] != 0 && delay_us == 0 {
                200
            } else {
                delay_us
            };
            line.push_str(&format!(" {} {}", steps[i], delay_us));
        }
        self.send_line(&line)?;
        self.expect_ok()?;
        for (acc, d) in self.steps.iter_mut().zip(steps.iter()) {
            *acc += *d;
        }
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;

    #[test]
    fn stepper_sends_protocol_and_updates_positions() {
        let mut t = MockTransport::new();
        t.push_reply("OK");
        let mut backend = StepperBackend::new(Box::new(t), 4);
        backend
            .move_steps(&[100, -50, 0, 25], &[0.001, 0.002, 0.0, 0.004])
            .unwrap();
        assert_eq!(backend.positions(), &[100, -50, 0, 25]);
    }
}
