//! ODrive ASCII-protocol backend.
//!
//! Streams `q <axis> <turns> [vel_lim]\n` after converting step deltas to turns.

use crate::transport::Transport;
use crate::{MotorBackend, Result, RuntimeError};

/// Per-axis ODrive configuration.
#[derive(Clone, Debug)]
pub struct ODriveAxis {
    /// ODrive axis index (0 or 1 on a single board).
    pub axis: u8,
    /// Steps per revolution used by the Player mapping.
    pub steps_per_rev: f64,
    /// Optional velocity limit in turns/s for `q`.
    pub velocity_lim: Option<f64>,
}

impl ODriveAxis {
    /// Axis helper.
    pub fn new(axis: u8, steps_per_rev: f64) -> Self {
        Self {
            axis,
            steps_per_rev,
            velocity_lim: Some(5.0),
        }
    }
}

/// ODrive backend over ASCII serial/TCP.
pub struct ODriveBackend {
    transport: Box<dyn Transport>,
    axes: Vec<ODriveAxis>,
    steps: Vec<i64>,
    turns: Vec<f64>,
}

impl ODriveBackend {
    /// Create backend for the given axis map.
    pub fn new(transport: Box<dyn Transport>, axes: Vec<ODriveAxis>) -> Self {
        let n = axes.len();
        Self {
            transport,
            axes,
            steps: vec![0; n],
            turns: vec![0.0; n],
        }
    }

    fn send_line(&mut self, line: &str) -> Result<()> {
        let mut msg = line.to_string();
        if !msg.ends_with('\n') {
            msg.push('\n');
        }
        self.transport.write_all(msg.as_bytes())
    }

    /// Enter closed-loop control (`requested_state = 8`).
    pub fn enter_closed_loop(&mut self) -> Result<()> {
        for a in self.axes.clone() {
            self.send_line(&format!("w axis{}.requested_state 8", a.axis))?;
        }
        Ok(())
    }

    /// Idle all axes (`requested_state = 1`).
    pub fn idle(&mut self) -> Result<()> {
        for a in self.axes.clone() {
            self.send_line(&format!("w axis{}.requested_state 1", a.axis))?;
        }
        Ok(())
    }

    /// Reset bookkeeping to zero turns/steps.
    pub fn zero_bookkeeping(&mut self) {
        for t in &mut self.turns {
            *t = 0.0;
        }
        for s in &mut self.steps {
            *s = 0;
        }
    }
}

impl MotorBackend for ODriveBackend {
    fn axis_count(&self) -> usize {
        self.axes.len()
    }

    fn move_steps(&mut self, steps: &[i64], _delays_s: &[f64]) -> Result<()> {
        if steps.len() != self.axes.len() {
            return Err(RuntimeError::Config("step vector length mismatch".into()));
        }
        for (i, &step) in steps.iter().enumerate() {
            let spr = self.axes[i].steps_per_rev;
            if spr <= 0.0 {
                return Err(RuntimeError::Config("steps_per_rev must be > 0".into()));
            }
            self.steps[i] += step;
            self.turns[i] += step as f64 / spr;
            let axis = self.axes[i].axis;
            let turns = self.turns[i];
            let line = if let Some(vlim) = self.axes[i].velocity_lim {
                format!("q {axis} {turns:.6} {vlim:.3}")
            } else {
                format!("q {axis} {turns:.6}")
            };
            self.send_line(&line)?;
        }
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.steps
    }

    fn read_feedback_steps(&mut self) -> Result<Vec<i64>> {
        let mut out = Vec::with_capacity(self.axes.len());
        for i in 0..self.axes.len() {
            let axis = self.axes[i].axis;
            self.send_line(&format!("f {axis}"))?;
            let resp = self.transport.read_line().unwrap_or_default();
            // response: "pos vel"
            let pos_turns: f64 = resp
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(self.turns[i]);
            self.turns[i] = pos_turns;
            let steps = (pos_turns * self.axes[i].steps_per_rev).round() as i64;
            self.steps[i] = steps;
            out.push(steps);
        }
        Ok(out)
    }

    fn estop(&mut self) -> Result<()> {
        self.idle()
    }

    fn home_hardware(&mut self) -> Result<()> {
        self.zero_bookkeeping();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;

    #[test]
    fn odrive_emits_q_commands_in_turns() {
        let t = MockTransport::new();
        let axes = vec![ODriveAxis::new(0, 200.0), ODriveAxis::new(1, 200.0)];
        let mut backend = ODriveBackend::new(Box::new(t), axes);
        backend.move_steps(&[200, -100], &[0.0, 0.0]).unwrap();
        assert_eq!(backend.positions(), &[200, -100]);
    }
}
