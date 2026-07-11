//! Simulation service helpers (workspace, trajectory, scene).

use nalgebra::{DVector, UnitQuaternion, Vector3};
use spyder_core::{FkOptions, Pose, Robot, Vec3};
use spyder_sim::{
    line_waypoints, sample_wrench_feasible, trajectory_lengths, SampleBox, SceneAnimation,
    SceneSnapshot, WorkspaceReport, write_scene_animation_html, write_scene_html,
};

use crate::dto::{
    FeasibleRequest, FeasibleResponse, FkRequest, FkResponse, IkRequest, IkResponse,
    JacobianRequest, JacobianResponse, SceneExportRequest, SceneExportResponse,
    SceneSnapshotRequest, SceneSnapshotResponse, TrajLineRequest, TrajLineResponse,
    TrajWaypointsRequest, TrajWaypointsResponse, WorkspaceRequest, WorkspaceResponse,
    WorkspaceSampleDto,
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
pub fn ik(
    robot: &Robot,
    req: &IkRequest,
    motor_axes: &[crate::dto::MotorAxisDto],
    reference_lengths: Option<&[f64]>,
) -> Result<IkResponse, String> {
    let mut robot = robot.clone();
    let params = model_params_for_ik(&robot, req);
    apply_cable_model(&mut robot, &params)?;
    let pose = Pose::from_position(Vec3::new(req.xyz[0], req.xyz[1], req.xyz[2]));
    let needs_wrench = params.model == "sag" || req.mg.is_some();

    let refs = req
        .reference_lengths
        .clone()
        .or_else(|| reference_lengths.map(|r| r.to_vec()));

    let mut opts = spyder_core::IkOptions::with_defaults();
    if needs_wrench {
        let mg = req.mg.unwrap_or(50.0);
        opts.wrench = Some(DVector::from_vec(vec![0.0, 0.0, -mg]));
        opts.f_min = 0.5;
        opts.f_max = 500.0;
    }
    if let Some(ref lens) = refs {
        if lens.len() == robot.anchors.len() {
            opts.reference_lengths = Some(lens.clone());
            let axes = crate::motor_svc::axes_from_state(motor_axes, robot.anchors.len())?;
            opts.winches = Some(axes.iter().map(|a| a.winch.clone()).collect());
            opts.motors = Some(axes.iter().map(|a| a.motor.clone()).collect());
        }
    }

    let result = if needs_wrench || opts.reference_lengths.is_some() {
        robot
            .ik_with_options(&pose, &opts)
            .map_err(|e| e.to_string())?
    } else {
        robot.ik(&pose).map_err(|e| e.to_string())?
    };

    let motor_commands = result.motor_commands.map(|cmds| {
        cmds.into_iter()
            .map(|c| crate::dto::MotorCommandDto {
                winch_radians: c.winch_radians,
                steps: c.steps,
                steps_exact: c.steps_exact,
            })
            .collect()
    });

    Ok(IkResponse {
        lengths: result.lengths,
        tensions: result.tensions,
        unstrained_lengths: if result.unstrained_lengths.iter().any(|u| u.is_some()) {
            Some(result.unstrained_lengths)
        } else {
            None
        },
        motor_commands,
    })
}

/// Forward kinematics from lengths.
pub fn fk(robot: &Robot, req: &FkRequest) -> Result<FkResponse, String> {
    let seed_pos = Vec3::new(req.seed[0], req.seed[1], req.seed[2]);
    let rv = req.orientation_rv.unwrap_or([0.0, 0.0, 0.0]);
    let orient = UnitQuaternion::from_scaled_axis(Vector3::new(rv[0], rv[1], rv[2]));
    let seed = Pose {
        position: seed_pos,
        orientation: orient,
    };
    let mut opts = if req.allow_underconstrained {
        FkOptions::permissive()
    } else {
        FkOptions::default()
    };
    if let Some(t) = &req.tensions {
        opts.tensions = Some(t.clone());
    }
    let result = robot
        .fk_with_options(&req.lengths, &seed, &opts)
        .map_err(|e| e.to_string())?;
    let out_rv = result.orientation.scaled_axis();
    Ok(FkResponse {
        xyz: [result.position.x, result.position.y, result.position.z],
        orientation_rv: [out_rv.x, out_rv.y, out_rv.z],
        method: format!("{:?}", result.method),
        residual: result.residual,
    })
}

fn pose_from_request(req_xyz: [f64; 3], orientation_rv: Option<[f64; 3]>) -> Pose {
    let position = Vec3::new(req_xyz[0], req_xyz[1], req_xyz[2]);
    if let Some(rv) = orientation_rv {
        Pose {
            position,
            orientation: UnitQuaternion::from_scaled_axis(Vector3::new(rv[0], rv[1], rv[2])),
        }
    } else {
        Pose::from_position(position)
    }
}

/// Length Jacobian at a pose.
pub fn jacobian(robot: &Robot, req: &JacobianRequest) -> Result<JacobianResponse, String> {
    let pose = pose_from_request(req.xyz, req.orientation_rv);
    if robot.point_mass {
        let j = robot.length_jacobian(&pose).map_err(|e| e.to_string())?;
        let rows: Vec<Vec<f64>> = (0..j.nrows())
            .map(|i| (0..j.ncols()).map(|c| j[(i, c)]).collect())
            .collect();
        Ok(JacobianResponse {
            cols: j.ncols(),
            rows,
        })
    } else {
        let j = robot.length_jacobian_6(&pose).map_err(|e| e.to_string())?;
        let rows: Vec<Vec<f64>> = (0..j.nrows())
            .map(|i| (0..j.ncols()).map(|c| j[(i, c)]).collect())
            .collect();
        Ok(JacobianResponse {
            cols: j.ncols(),
            rows,
        })
    }
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
    traj_from_waypoints(robot, &waypoints)
}

/// IK lengths for an arbitrary waypoint list.
pub fn traj_waypoints(
    robot: &Robot,
    req: &TrajWaypointsRequest,
) -> Result<TrajWaypointsResponse, String> {
    if req.waypoints.len() < 2 {
        return Err("need at least 2 waypoints".into());
    }
    let waypoints: Vec<Vec3> = req
        .waypoints
        .iter()
        .map(|w| Vec3::new(w[0], w[1], w[2]))
        .collect();
    let resp = traj_from_waypoints(robot, &waypoints)?;
    Ok(TrajWaypointsResponse {
        waypoints: resp.waypoints,
        lengths: resp.lengths,
    })
}

fn traj_from_waypoints(robot: &Robot, waypoints: &[Vec3]) -> Result<TrajLineResponse, String> {
    let lengths = trajectory_lengths(robot, waypoints).map_err(|e| e.to_string())?;
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
    let pose = pose_from_request(req.xyz, req.orientation_rv);
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

/// Export Plotly HTML for current pose (or animation).
pub fn scene_export(robot: &Robot, req: &SceneExportRequest) -> Result<SceneExportResponse, String> {
    let pose = pose_from_request(req.xyz, req.orientation_rv);
    let path = std::env::temp_dir().join(format!("spyder_export_{}.html", std::process::id()));
    if req.format == "html_anim" {
        let waypoints: Vec<Vec3> = if let Some(wps) = &req.waypoints {
            wps.iter()
                .map(|w| Vec3::new(w[0], w[1], w[2]))
                .collect()
        } else {
            line_waypoints(pose.position, pose.position, 1)
        };
        let mut frames = Vec::new();
        for wp in &waypoints {
            let p = Pose {
                position: *wp,
                orientation: pose.orientation,
            };
            frames.push(SceneSnapshot::from_robot(robot, &p).map_err(|e| e.to_string())?);
        }
        let anim = SceneAnimation {
            anchors: frames[0].anchors.clone(),
            frames,
            workspace: None,
        };
        write_scene_animation_html(&anim, &path, "Spyder scene")
            .map_err(|e| e.to_string())?;
    } else {
        let snap = SceneSnapshot::from_robot(robot, &pose).map_err(|e| e.to_string())?;
        write_scene_html(&snap, &path, "Spyder scene").map_err(|e| e.to_string())?;
    }
    let html = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&path);
    Ok(SceneExportResponse { html })
}
