//! Simulation service helpers (workspace, trajectory, scene).

use nalgebra::DVector;
use spyder_core::{Pose, Robot, Vec3};
use spyder_sim::{
    line_waypoints, sample_wrench_feasible, trajectory_lengths, SampleBox, SceneSnapshot,
    WorkspaceReport,
};

use crate::dto::{
    FeasibleRequest, FeasibleResponse, FkRequest, FkResponse, IkRequest, IkResponse,
    JacobianRequest, JacobianResponse, SceneSnapshotRequest, SceneSnapshotResponse,
    TrajLineRequest, TrajLineResponse, WorkspaceRequest, WorkspaceResponse, WorkspaceSampleDto,
};
use crate::state::{apply_cable_model, cable_model_params, CableModelParams};

fn model_params_for_ik(robot: &Robot, req: &IkRequest) -> CableModelParams {
    let base = cable_model_params(robot);
    if let Some(ref model) = req.model {
        CableModelParams {
            model: model.clone(),
            ..base
        }
    } else {
        base
    }
}

/// Inverse kinematics at a pose.
pub fn ik(robot: &Robot, req: &IkRequest) -> Result<IkResponse, String> {
    let mut robot = robot.clone();
    let params = model_params_for_ik(&robot, req);
    apply_cable_model(&mut robot, &params)?;
    let pose = Pose::from_position(Vec3::new(req.xyz[0], req.xyz[1], req.xyz[2]));
    let needs_wrench = params.model == "sag" || req.mg.is_some();
    let result = if needs_wrench {
        let mg = req.mg.unwrap_or(50.0);
        let mut opts = spyder_core::IkOptions::with_defaults();
        opts.wrench = Some(DVector::from_vec(vec![0.0, 0.0, -mg]));
        opts.f_min = 0.5;
        opts.f_max = 500.0;
        robot.ik_with_options(&pose, &opts).map_err(|e| e.to_string())?
    } else {
        robot.ik(&pose).map_err(|e| e.to_string())?
    };
    Ok(IkResponse {
        lengths: result.lengths,
        tensions: result.tensions,
        unstrained_lengths: if result.unstrained_lengths.iter().any(|u| u.is_some()) {
            Some(result.unstrained_lengths)
        } else {
            None
        },
    })
}

/// Forward kinematics from lengths.
pub fn fk(robot: &Robot, req: &FkRequest) -> Result<FkResponse, String> {
    let seed = Vec3::new(req.seed[0], req.seed[1], req.seed[2]);
    let result = robot.fk(&req.lengths, seed).map_err(|e| e.to_string())?;
    let rv = result.orientation.scaled_axis();
    Ok(FkResponse {
        xyz: [result.position.x, result.position.y, result.position.z],
        orientation_rv: [rv.x, rv.y, rv.z],
        method: format!("{:?}", result.method),
        residual: result.residual,
    })
}

/// Length Jacobian at a pose.
pub fn jacobian(robot: &Robot, req: &JacobianRequest) -> Result<JacobianResponse, String> {
    let pose = Pose::from_position(Vec3::new(req.xyz[0], req.xyz[1], req.xyz[2]));
    let j = robot.length_jacobian(&pose).map_err(|e| e.to_string())?;
    let rows: Vec<[f64; 3]> = (0..j.nrows())
        .map(|i| [j[(i, 0)], j[(i, 1)], j[(i, 2)]])
        .collect();
    Ok(JacobianResponse { rows })
}

/// Wrench feasibility check.
pub fn feasible(robot: &Robot, req: &FeasibleRequest) -> Result<FeasibleResponse, String> {
    let pose = Pose::from_position(Vec3::new(req.xyz[0], req.xyz[1], req.xyz[2]));
    let wrench = DVector::from_vec(vec![0.0, 0.0, -req.mg]);
    let ok = robot
        .is_wrench_feasible(&pose, wrench, req.f_min, req.f_max)
        .map_err(|e| e.to_string())?;
    Ok(FeasibleResponse { ok })
}

/// Sample wrench-feasible workspace.
pub fn workspace(robot: &Robot, req: &WorkspaceRequest) -> WorkspaceResponse {
    let box_ = SampleBox {
        min: Vec3::new(req.min[0], req.min[1], req.min[2]),
        max: Vec3::new(req.max[0], req.max[1], req.max[2]),
        nx: req.nx,
        ny: req.ny,
        nz: req.nz,
    };
    let wrench = DVector::from_vec(vec![0.0, 0.0, -req.mg]);
    let report = workspace_report(robot, &box_, wrench, req.f_min, req.f_max);
    workspace_to_dto(&report)
}

fn workspace_report(
    robot: &Robot,
    box_: &SampleBox,
    wrench: DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> WorkspaceReport {
    sample_wrench_feasible(robot, box_, wrench, f_min, f_max)
}

fn workspace_to_dto(report: &WorkspaceReport) -> WorkspaceResponse {
    WorkspaceResponse {
        fraction: report.fraction,
        samples: report
            .samples
            .iter()
            .map(|s| WorkspaceSampleDto {
                x: s.x,
                y: s.y,
                z: s.z,
                feasible: s.feasible,
            })
            .collect(),
    }
}

/// Cartesian line trajectory with IK lengths.
pub fn traj_line(robot: &Robot, req: &TrajLineRequest) -> Result<TrajLineResponse, String> {
    let start = Vec3::new(req.start[0], req.start[1], req.start[2]);
    let end = Vec3::new(req.end[0], req.end[1], req.end[2]);
    let waypoints = line_waypoints(start, end, req.segments);
    let lengths = trajectory_lengths(robot, &waypoints).map_err(|e| e.to_string())?;
    Ok(TrajLineResponse {
        waypoints: waypoints
            .iter()
            .map(|p| [p.x, p.y, p.z])
            .collect(),
        lengths,
    })
}

/// 3D scene snapshot at a pose (model-aware cable paths).
pub fn scene_snapshot(
    robot: &Robot,
    req: &SceneSnapshotRequest,
) -> Result<SceneSnapshotResponse, String> {
    let pose = Pose::from_position(Vec3::new(req.xyz[0], req.xyz[1], req.xyz[2]));
    let snap = SceneSnapshot::from_robot(robot, &pose).map_err(|e| e.to_string())?;
    Ok(SceneSnapshotResponse {
        anchors: snap.anchors,
        dolly: snap.dolly,
        attachments: snap.attachments,
        lengths: snap.lengths,
        cable_paths: snap.cable_paths,
        unit_pulls: snap.unit_pulls,
        model: snap.model,
    })
}
