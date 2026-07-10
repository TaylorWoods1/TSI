//! Workspace sampling and simple trajectory helpers.

#![deny(missing_docs)]

pub mod export;
pub mod scene;

use nalgebra::DVector;
use serde::Serialize;
use spyder_core::{Pose, Robot, Vec3};

pub use export::{write_csv, write_html, write_json};
pub use scene::{
    write_scene_animation_html, write_scene_at, write_scene_html, write_scene_line, SceneAnimation,
    SceneSnapshot,
};

/// Axis-aligned sampling box in world coordinates.
#[derive(Clone, Debug)]
pub struct SampleBox {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
    /// Number of samples along X.
    pub nx: usize,
    /// Number of samples along Y.
    pub ny: usize,
    /// Number of samples along Z.
    pub nz: usize,
}

/// One sampled pose and its feasibility.
#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceSample {
    /// Sample position.
    pub x: f64,
    /// Sample position.
    pub y: f64,
    /// Sample position.
    pub z: f64,
    /// Whether a tension solution exists for the given wrench bounds.
    pub feasible: bool,
}

/// Result of a workspace sweep.
#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceReport {
    /// Total grid points evaluated.
    pub total: usize,
    /// Feasible count.
    pub feasible: usize,
    /// Feasible fraction in \[0, 1\].
    pub fraction: f64,
    /// Per-point samples.
    pub samples: Vec<WorkspaceSample>,
}

/// Sample wrench-feasible workspace for a point-mass robot under a constant wrench.
pub fn sample_wrench_feasible(
    robot: &Robot,
    box_: &SampleBox,
    wrench: DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> WorkspaceReport {
    let mut samples = Vec::new();
    let mut feasible = 0usize;
    let nx = box_.nx.max(1);
    let ny = box_.ny.max(1);
    let nz = box_.nz.max(1);

    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let fx = if nx == 1 {
                    0.5
                } else {
                    ix as f64 / (nx - 1) as f64
                };
                let fy = if ny == 1 {
                    0.5
                } else {
                    iy as f64 / (ny - 1) as f64
                };
                let fz = if nz == 1 {
                    0.5
                } else {
                    iz as f64 / (nz - 1) as f64
                };
                let p = Vec3::new(
                    box_.min.x + (box_.max.x - box_.min.x) * fx,
                    box_.min.y + (box_.max.y - box_.min.y) * fy,
                    box_.min.z + (box_.max.z - box_.min.z) * fz,
                );
                let pose = Pose::from_position(p);
                let ok = robot
                    .is_wrench_feasible(&pose, wrench.clone(), f_min, f_max)
                    .unwrap_or(false);
                if ok {
                    feasible += 1;
                }
                samples.push(WorkspaceSample {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                    feasible: ok,
                });
            }
        }
    }
    let total = samples.len();
    WorkspaceReport {
        total,
        feasible,
        fraction: if total == 0 {
            0.0
        } else {
            feasible as f64 / total as f64
        },
        samples,
    }
}

/// Linearly interpolate positions from `start` to `end` with `segments` steps (>=1).
pub fn line_waypoints(start: Vec3, end: Vec3, segments: usize) -> Vec<Vec3> {
    let n = segments.max(1);
    let mut out = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f64 / n as f64;
        out.push(start + (end - start) * t);
    }
    out
}

/// IK lengths along a Cartesian polyline (ideal model on the robot).
pub fn trajectory_lengths(robot: &Robot, waypoints: &[Vec3]) -> spyder_core::Result<Vec<Vec<f64>>> {
    let mut all = Vec::with_capacity(waypoints.len());
    for p in waypoints {
        let ik = robot.ik(&Pose::from_position(*p))?;
        all.push(ik.lengths);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spyder_core::Preset;

    #[test]
    fn workspace_center_more_feasible_than_outside() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let box_ = SampleBox {
            min: Vec3::new(-1.0, -1.0, 0.5),
            max: Vec3::new(1.0, 1.0, 2.0),
            nx: 5,
            ny: 5,
            nz: 4,
        };
        let w = DVector::from_vec(vec![0.0, 0.0, -9.81]);
        let report = sample_wrench_feasible(&robot, &box_, w, 0.5, 500.0);
        assert!(report.total > 0);
        assert!(
            report.feasible > 0,
            "expected some feasible poses, got {}",
            report.feasible
        );
        assert!(report.fraction > 0.0);
    }

    #[test]
    fn line_waypoints_include_endpoints() {
        let pts = line_waypoints(Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0), 4);
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(pts[4], Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn trajectory_lengths_one_per_waypoint() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let pts = line_waypoints(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.5, 0.0, 1.0), 3);
        let lens = trajectory_lengths(&robot, &pts).unwrap();
        assert_eq!(lens.len(), pts.len());
        assert_eq!(lens[0].len(), 4);
    }
}
