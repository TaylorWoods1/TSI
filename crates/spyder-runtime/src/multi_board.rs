//! Fan-out motor backend across multiple boards via [`AxisMap`].

use crate::axis_map::AxisMap;
use crate::{MockBackend, MotorBackend, Result, RuntimeError};

/// Route from global cable index → board + local axis.
#[derive(Clone, Debug)]
struct CableRoute {
    board: usize,
    local: usize,
}

/// Composite backend: one [`MotorBackend`] per unique device in an [`AxisMap`].
pub struct MultiBoardBackend {
    map: AxisMap,
    boards: Vec<Box<dyn MotorBackend>>,
    routes: Vec<CableRoute>,
    positions: Vec<i64>,
}

impl MultiBoardBackend {
    /// Build from an axis map and already-opened per-device backends.
    ///
    /// `boards` must be ordered to match [`AxisMap::devices`] (first-seen device order).
    /// Each board's `axis_count` must equal the number of cables mapped to that device.
    pub fn new(map: AxisMap, boards: Vec<Box<dyn MotorBackend>>) -> Result<Self> {
        map.validate()?;
        let devices = map.devices();
        if boards.len() != devices.len() {
            return Err(RuntimeError::Config(format!(
                "expected {} boards for axis map, got {}",
                devices.len(),
                boards.len()
            )));
        }

        // Per device: cables sorted by hardware axis index → local 0..k
        let mut routes = vec![
            CableRoute {
                board: 0,
                local: 0
            };
            map.cables.len()
        ];
        for (board_idx, device) in devices.iter().enumerate() {
            let mut cable_idxs: Vec<usize> = map
                .cables
                .iter()
                .enumerate()
                .filter(|(_, c)| &c.device == device)
                .map(|(i, _)| i)
                .collect();
            cable_idxs.sort_by_key(|&i| map.cables[i].axis);
            if boards[board_idx].axis_count() != cable_idxs.len() {
                return Err(RuntimeError::Config(format!(
                    "board {board_idx} ({device}) axis_count={} but map has {} cables",
                    boards[board_idx].axis_count(),
                    cable_idxs.len()
                )));
            }
            for (local, &cable) in cable_idxs.iter().enumerate() {
                routes[cable] = CableRoute {
                    board: board_idx,
                    local,
                };
            }
        }

        Ok(Self {
            positions: vec![0; map.cables.len()],
            map,
            boards,
            routes,
        })
    }

    /// Dry-run: one [`MockBackend`] per device.
    pub fn mock_from_map(map: AxisMap) -> Result<Self> {
        let devices = map.devices();
        let mut boards: Vec<Box<dyn MotorBackend>> = Vec::with_capacity(devices.len());
        for device in &devices {
            let n = map.cables.iter().filter(|c| &c.device == device).count();
            boards.push(Box::new(MockBackend::new(n)));
        }
        Self::new(map, boards)
    }

    /// Underlying axis map.
    pub fn map(&self) -> &AxisMap {
        &self.map
    }

    /// Number of physical boards.
    pub fn board_count(&self) -> usize {
        self.boards.len()
    }
}

impl MotorBackend for MultiBoardBackend {
    fn axis_count(&self) -> usize {
        self.positions.len()
    }

    fn move_steps(&mut self, steps: &[i64], delays_s: &[f64]) -> Result<()> {
        if steps.len() != self.positions.len() || delays_s.len() != self.positions.len() {
            return Err(RuntimeError::Config(
                "step/delay length mismatch for multi-board".into(),
            ));
        }
        // Gather per-board command vectors.
        let mut board_steps: Vec<Vec<i64>> = self
            .boards
            .iter()
            .map(|b| vec![0; b.axis_count()])
            .collect();
        let mut board_delays: Vec<Vec<f64>> = self
            .boards
            .iter()
            .map(|b| vec![0.0; b.axis_count()])
            .collect();
        for (cable, route) in self.routes.iter().enumerate() {
            board_steps[route.board][route.local] = steps[cable];
            board_delays[route.board][route.local] = delays_s[cable];
        }
        for (i, board) in self.boards.iter_mut().enumerate() {
            board.move_steps(&board_steps[i], &board_delays[i])?;
        }
        for i in 0..self.positions.len() {
            self.positions[i] += steps[i];
        }
        Ok(())
    }

    fn positions(&self) -> &[i64] {
        &self.positions
    }

    fn read_feedback_steps(&mut self) -> Result<Vec<i64>> {
        let mut out = vec![0i64; self.positions.len()];
        let mut board_fb: Vec<Vec<i64>> = Vec::with_capacity(self.boards.len());
        for board in self.boards.iter_mut() {
            board_fb.push(board.read_feedback_steps()?);
        }
        for (cable, route) in self.routes.iter().enumerate() {
            out[cable] = board_fb[route.board][route.local];
            self.positions[cable] = out[cable];
        }
        Ok(out)
    }

    fn estop(&mut self) -> Result<()> {
        for board in &mut self.boards {
            board.estop()?;
        }
        Ok(())
    }

    fn home_hardware(&mut self) -> Result<()> {
        for board in &mut self.boards {
            board.home_hardware()?;
        }
        for p in &mut self.positions {
            *p = 0;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis_map::{AxisEndpoint, AxisMap};

    fn dual_mock_map() -> AxisMap {
        AxisMap {
            cables: vec![
                AxisEndpoint {
                    device: "board-a".into(),
                    baud: 115200,
                    axis: 0,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "board-a".into(),
                    baud: 115200,
                    axis: 1,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "board-b".into(),
                    baud: 115200,
                    axis: 0,
                    steps_per_rev: 200.0,
                },
                AxisEndpoint {
                    device: "board-b".into(),
                    baud: 115200,
                    axis: 1,
                    steps_per_rev: 200.0,
                },
            ],
        }
    }

    #[test]
    fn fans_out_steps_to_two_mock_boards() {
        let map = dual_mock_map();
        let mut backend = MultiBoardBackend::mock_from_map(map).unwrap();
        assert_eq!(backend.board_count(), 2);
        backend
            .move_steps(&[10, -20, 30, -40], &[0.01, 0.01, 0.01, 0.01])
            .unwrap();
        assert_eq!(backend.positions(), &[10, -20, 30, -40]);
        let fb = backend.read_feedback_steps().unwrap();
        assert_eq!(fb, vec![10, -20, 30, -40]);
    }

    #[test]
    fn player_works_with_multiboard_mock() {
        use crate::{Axis, Player};
        use spyder_core::{Preset, Robot, Vec3};

        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let map = AxisMap::example_dual_odrive();
        let backend = MultiBoardBackend::mock_from_map(map).unwrap();
        let axes: Vec<_> = (0..4)
            .map(|_| Axis::new(0.05, 200.0, 1.0).unwrap())
            .collect();
        let home = Vec3::new(0.0, 0.0, 1.0);
        let mut player = Player::new(&robot, axes, backend, home).unwrap();
        player
            .move_line(home, Vec3::new(0.3, 0.0, 1.0), 4, 1.0)
            .unwrap();
        assert!(player.backend.positions().iter().any(|s| *s != 0));
    }
}
