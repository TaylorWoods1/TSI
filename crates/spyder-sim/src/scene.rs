//! 3D scene export: anchors, cables, dolly — static + animated trajectories.

use std::fs;
use std::path::Path;

use serde::Serialize;
use spyder_core::{
    cable_paths_at_pose, unit_pulls_at_pose, Pose, Robot, Vec3,
};
use spyder_core::cable_eval::default_pulley_radius;

use crate::{line_waypoints, WorkspaceReport};

/// Snapshot of the robot at a pose for visualization.
#[derive(Clone, Debug, Serialize)]
pub struct SceneSnapshot {
    /// Anchor world positions (pulley centers).
    pub anchors: Vec<[f64; 3]>,
    /// Dolly / EE position.
    pub dolly: [f64; 3],
    /// Cable lengths.
    pub lengths: Vec<f64>,
    /// Attachment world points.
    pub attachments: Vec<[f64; 3]>,
    /// Model-aware cable polylines (one vertex list per cable).
    pub cable_paths: Vec<Vec<[f64; 3]>>,
    /// Unit pull directions at each attachment (for force arrows).
    pub unit_pulls: Vec<[f64; 3]>,
    /// Active cable model label.
    pub model: String,
}

impl SceneSnapshot {
    /// Build from robot + pose via IK (model-aware cable paths).
    pub fn from_robot(robot: &Robot, pose: &Pose) -> spyder_core::Result<Self> {
        use nalgebra::DVector;
        use spyder_core::{CableModelKind, IkOptions};

        let ik = match &robot.cable_model {
            CableModelKind::Sag(_) => {
                let mut opts = IkOptions::with_defaults();
                opts.wrench = Some(DVector::from_vec(vec![0.0, 0.0, -50.0]));
                opts.f_min = 1.0;
                opts.f_max = 1.0e4;
                robot.ik_with_options(pose, &opts)?
            }
            _ => robot.ik(pose)?,
        };
        let attachments_body = if robot.point_mass {
            robot
                .anchors
                .iter()
                .map(|_| spyder_core::PlatformAttachment::origin())
                .collect::<Vec<_>>()
        } else {
            robot.attachments.clone()
        };
        let anchors: Vec<[f64; 3]> = robot
            .anchors
            .iter()
            .map(|a| [a.exit.x, a.exit.y, a.exit.z])
            .collect();
        let attachments: Vec<[f64; 3]> = attachments_body
            .iter()
            .map(|att| {
                let p = pose.transform_point(&att.body_point);
                [p.x, p.y, p.z]
            })
            .collect();
        let cable_paths = cable_paths_at_pose(
            &robot.anchors,
            &attachments_body,
            pose,
            &robot.cable_model,
            ik.tensions.as_deref(),
        )?;
        let def_r = default_pulley_radius(&robot.cable_model);
        let pulls = unit_pulls_at_pose(
            &robot.anchors,
            &attachments_body,
            pose,
            &robot.cable_model,
            ik.tensions.as_deref(),
            def_r,
        )?;
        let unit_pulls: Vec<[f64; 3]> = pulls.iter().map(|u| [u.x, u.y, u.z]).collect();
        let model = match &robot.cable_model {
            spyder_core::CableModelKind::Ideal => "ideal".into(),
            spyder_core::CableModelKind::Pulley { .. } => "pulley".into(),
            spyder_core::CableModelKind::Sag(_) => "sag".into(),
        };
        Ok(Self {
            anchors,
            dolly: [pose.position.x, pose.position.y, pose.position.z],
            lengths: ik.lengths,
            attachments,
            cable_paths,
            unit_pulls,
            model,
        })
    }
}

/// Multi-frame scene for trajectory playback.
#[derive(Clone, Debug, Serialize)]
pub struct SceneAnimation {
    /// Shared anchors.
    pub anchors: Vec<[f64; 3]>,
    /// One snapshot per waypoint (attachments + dolly + lengths).
    pub frames: Vec<SceneSnapshot>,
    /// Optional workspace cloud (feasible samples only).
    pub workspace: Option<Vec<[f64; 3]>>,
}

impl SceneAnimation {
    /// Build frames along Cartesian waypoints.
    pub fn from_waypoints(robot: &Robot, waypoints: &[Vec3]) -> spyder_core::Result<Self> {
        if waypoints.is_empty() {
            return Err(spyder_core::SpyderError::Config(
                "need at least one waypoint".into(),
            ));
        }
        let mut frames = Vec::with_capacity(waypoints.len());
        for p in waypoints {
            frames.push(SceneSnapshot::from_robot(
                robot,
                &Pose::from_position(*p),
            )?);
        }
        Ok(Self {
            anchors: frames[0].anchors.clone(),
            frames,
            workspace: None,
        })
    }

    /// Straight-line animation from `start` to `end`.
    pub fn from_line(
        robot: &Robot,
        start: Vec3,
        end: Vec3,
        segments: usize,
    ) -> spyder_core::Result<Self> {
        let pts = line_waypoints(start, end, segments);
        Self::from_waypoints(robot, &pts)
    }

    /// Attach feasible workspace samples as a background cloud.
    pub fn with_workspace(mut self, report: &WorkspaceReport) -> Self {
        let pts: Vec<[f64; 3]> = report
            .samples
            .iter()
            .filter(|s| s.feasible)
            .map(|s| [s.x, s.y, s.z])
            .collect();
        self.workspace = Some(pts);
        self
    }
}

fn cable_js_for_frame(scene: &SceneSnapshot) -> String {
    let mut cable_traces = String::new();
    for (i, path) in scene.cable_paths.iter().enumerate() {
        if i > 0 {
            cable_traces.push(',');
        }
        if path.len() < 2 {
            continue;
        }
        let xs: Vec<f64> = path.iter().map(|p| p[0]).collect();
        let ys: Vec<f64> = path.iter().map(|p| p[1]).collect();
        let zs: Vec<f64> = path.iter().map(|p| p[2]).collect();
        cable_traces.push_str(&format!(
            r#"{{
      x: {xs}, y: {ys}, z: {zs},
      mode: 'lines', type: 'scatter3d', name: 'cable {i}',
      line: {{ width: 6, color: '#f0a202' }},
      showlegend: false
    }}"#,
            xs = serde_json::to_string(&xs).unwrap(),
            ys = serde_json::to_string(&ys).unwrap(),
            zs = serde_json::to_string(&zs).unwrap(),
        ));
    }
    cable_traces
}

/// Write interactive Plotly HTML with anchors, cables, and dolly.
pub fn write_scene_html(scene: &SceneSnapshot, path: &Path, title: &str) -> std::io::Result<()> {
    let cable_traces = cable_js_for_frame(scene);
    let ax: Vec<f64> = scene.anchors.iter().map(|a| a[0]).collect();
    let ay: Vec<f64> = scene.anchors.iter().map(|a| a[1]).collect();
    let az: Vec<f64> = scene.anchors.iter().map(|a| a[2]).collect();
    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
  <style>
    body {{ margin:0; background:#0b1020; color:#e8eefc; font-family: ui-sans-serif, system-ui, sans-serif; }}
    header {{ padding: 1rem 1.25rem; border-bottom: 1px solid #243056; }}
    h1 {{ margin:0; font-size:1.05rem; }}
    .meta {{ opacity:.7; font-size:.85rem; margin-top:.3rem; }}
    #plot {{ width:100vw; height: calc(100vh - 72px); }}
  </style>
</head>
<body>
  <header>
    <h1>{title}</h1>
    <div class="meta">dolly [{dx:.3}, {dy:.3}, {dz:.3}] · {n} cables · lengths [{lengths}]</div>
  </header>
  <div id="plot"></div>
  <script>
    const anchors = {{
      x: {ax}, y: {ay}, z: {az},
      mode: 'markers+text', type: 'scatter3d', name: 'anchors',
      text: {labels},
      marker: {{ size: 5, color: '#6ea8fe' }},
      textposition: 'top center'
    }};
    const dolly = {{
      x: [{dx}], y: [{dy}], z: [{dz}],
      mode: 'markers', type: 'scatter3d', name: 'dolly',
      marker: {{ size: 8, color: '#ff6b6b', symbol: 'diamond' }}
    }};
    const cables = [
      {cable_traces}
    ];
    Plotly.newPlot('plot', [anchors, dolly, ...cables], {{
      paper_bgcolor: '#0b1020',
      plot_bgcolor: '#0b1020',
      font: {{ color: '#e8eefc' }},
      scene: {{
        xaxis: {{ title: 'X (m)' }},
        yaxis: {{ title: 'Y (m)' }},
        zaxis: {{ title: 'Z (m)' }},
        aspectmode: 'data'
      }},
      margin: {{ l:0,r:0,t:0,b:0 }}
    }}, {{responsive:true}});
  </script>
</body>
</html>
"##,
        title = title,
        dx = scene.dolly[0],
        dy = scene.dolly[1],
        dz = scene.dolly[2],
        n = scene.anchors.len(),
        lengths = scene
            .lengths
            .iter()
            .map(|l| format!("{l:.2}"))
            .collect::<Vec<_>>()
            .join(", "),
        ax = serde_json::to_string(&ax).unwrap(),
        ay = serde_json::to_string(&ay).unwrap(),
        az = serde_json::to_string(&az).unwrap(),
        labels = serde_json::to_string(
            &(0..scene.anchors.len())
                .map(|i| format!("A{i}"))
                .collect::<Vec<_>>()
        )
        .unwrap(),
        cable_traces = cable_traces,
    );
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, html)
}

/// Write animated Plotly HTML with play/pause + frame slider.
pub fn write_scene_animation_html(
    anim: &SceneAnimation,
    path: &Path,
    title: &str,
) -> std::io::Result<()> {
    let n_cables = anim.anchors.len();
    let ax: Vec<f64> = anim.anchors.iter().map(|a| a[0]).collect();
    let ay: Vec<f64> = anim.anchors.iter().map(|a| a[1]).collect();
    let az: Vec<f64> = anim.anchors.iter().map(|a| a[2]).collect();
    let labels: Vec<String> = (0..n_cables).map(|i| format!("A{i}")).collect();

    let mut frames_js = String::from("[");
    for (fi, frame) in anim.frames.iter().enumerate() {
        if fi > 0 {
            frames_js.push(',');
        }
        let mut data = String::from("[");
        // index 0 anchors (unchanged — still include for Plotly frame completeness)
        data.push_str(&format!(
            "{{x:{ax},y:{ay},z:{az}}}",
            ax = serde_json::to_string(&ax).unwrap(),
            ay = serde_json::to_string(&ay).unwrap(),
            az = serde_json::to_string(&az).unwrap(),
        ));
        // index 1 dolly
        data.push_str(&format!(
            ",{{x:[{}],y:[{}],z:[{}]}}",
            frame.dolly[0], frame.dolly[1], frame.dolly[2]
        ));
        // cables start at index 2
        for path in &frame.cable_paths {
            if path.len() < 2 {
                continue;
            }
            let xs: Vec<f64> = path.iter().map(|p| p[0]).collect();
            let ys: Vec<f64> = path.iter().map(|p| p[1]).collect();
            let zs: Vec<f64> = path.iter().map(|p| p[2]).collect();
            data.push_str(&format!(
                ",{{x:{xs},y:{ys},z:{zs}}}",
                xs = serde_json::to_string(&xs).unwrap(),
                ys = serde_json::to_string(&ys).unwrap(),
                zs = serde_json::to_string(&zs).unwrap(),
            ));
        }
        // optional workspace at end — static, omit from frame updates
        data.push(']');
        let meta = format!(
            "frame {fi}/{} · dolly [{:.2},{:.2},{:.2}]",
            anim.frames.len() - 1,
            frame.dolly[0],
            frame.dolly[1],
            frame.dolly[2]
        );
        frames_js.push_str(&format!(
            "{{name:'f{fi}',data:{data},traces:{traces}}}",
            traces = serde_json::to_string(
                &(0..(2 + n_cables)).collect::<Vec<_>>()
            )
            .unwrap(),
        ));
        let _ = meta; // used in slider steps below
    }
    frames_js.push(']');

    let mut slider_steps = String::from("[");
    for fi in 0..anim.frames.len() {
        if fi > 0 {
            slider_steps.push(',');
        }
        let frame = &anim.frames[fi];
        let label = format!("{fi}");
        let meta = format!(
            "frame {fi} · [{:.2},{:.2},{:.2}]",
            frame.dolly[0], frame.dolly[1], frame.dolly[2]
        );
        slider_steps.push_str(&format!(
            r#"{{
        method: 'animate',
        label: '{label}',
        args: [['f{fi}'], {{mode:'immediate',frame:{{duration:0,redraw:true}},transition:{{duration:0}}}}],
        execute: true
      }}"#
        ));
        let _ = meta;
    }
    slider_steps.push(']');

    let first = &anim.frames[0];
    let mut init_cables = String::new();
    for (i, path) in first.cable_paths.iter().enumerate() {
        if path.len() < 2 {
            continue;
        }
        if i > 0 {
            init_cables.push(',');
        }
        let xs: Vec<f64> = path.iter().map(|p| p[0]).collect();
        let ys: Vec<f64> = path.iter().map(|p| p[1]).collect();
        let zs: Vec<f64> = path.iter().map(|p| p[2]).collect();
        init_cables.push_str(&format!(
            r#"{{
      x: {xs}, y: {ys}, z: {zs},
      mode: 'lines', type: 'scatter3d', name: 'cable {i}',
      line: {{ width: 6, color: '#f0a202' }},
      showlegend: false
    }}"#,
            xs = serde_json::to_string(&xs).unwrap(),
            ys = serde_json::to_string(&ys).unwrap(),
            zs = serde_json::to_string(&zs).unwrap(),
        ));
    }

    let workspace_trace = if let Some(ref pts) = anim.workspace {
        if pts.is_empty() {
            "null".into()
        } else {
            let wx: Vec<f64> = pts.iter().map(|p| p[0]).collect();
            let wy: Vec<f64> = pts.iter().map(|p| p[1]).collect();
            let wz: Vec<f64> = pts.iter().map(|p| p[2]).collect();
            format!(
                r#"{{
      x: {wx}, y: {wy}, z: {wz},
      mode: 'markers', type: 'scatter3d', name: 'workspace',
      marker: {{ size: 2, color: '#3d5a80', opacity: 0.35 }},
      hoverinfo: 'skip'
    }}"#,
                wx = serde_json::to_string(&wx).unwrap(),
                wy = serde_json::to_string(&wy).unwrap(),
                wz = serde_json::to_string(&wz).unwrap(),
            )
        }
    } else {
        "null".into()
    };

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
  <style>
    body {{ margin:0; background:#0b1020; color:#e8eefc; font-family: ui-sans-serif, system-ui, sans-serif; }}
    header {{ padding: 1rem 1.25rem; border-bottom: 1px solid #243056; }}
    h1 {{ margin:0; font-size:1.05rem; }}
    .meta {{ opacity:.7; font-size:.85rem; margin-top:.3rem; }}
    #plot {{ width:100vw; height: calc(100vh - 72px); }}
  </style>
</head>
<body>
  <header>
    <h1>{title}</h1>
    <div class="meta" id="meta">{n_frames} frames · {n_cables} cables · play / scrub below</div>
  </header>
  <div id="plot"></div>
  <script>
    const anchors = {{
      x: {ax}, y: {ay}, z: {az},
      mode: 'markers+text', type: 'scatter3d', name: 'anchors',
      text: {labels},
      marker: {{ size: 5, color: '#6ea8fe' }},
      textposition: 'top center'
    }};
    const dolly = {{
      x: [{dx}], y: [{dy}], z: [{dz}],
      mode: 'markers', type: 'scatter3d', name: 'dolly',
      marker: {{ size: 8, color: '#ff6b6b', symbol: 'diamond' }}
    }};
    const cables = [{init_cables}];
    const workspace = {workspace_trace};
    const data = workspace ? [anchors, dolly, ...cables, workspace] : [anchors, dolly, ...cables];
    const frames = {frames_js};
    const layout = {{
      paper_bgcolor: '#0b1020',
      plot_bgcolor: '#0b1020',
      font: {{ color: '#e8eefc' }},
      scene: {{
        xaxis: {{ title: 'X (m)' }},
        yaxis: {{ title: 'Y (m)' }},
        zaxis: {{ title: 'Z (m)' }},
        aspectmode: 'data'
      }},
      margin: {{ l:0,r:0,t:0,b:0 }},
      sliders: [{{
        active: 0,
        pad: {{ t: 30, b: 10 }},
        currentvalue: {{ prefix: 'frame: ', font: {{ color: '#e8eefc' }} }},
        steps: {slider_steps}
      }}],
      updatemenus: [{{
        type: 'buttons',
        showactive: false,
        y: 0,
        x: 0.05,
        xanchor: 'left',
        yanchor: 'top',
        pad: {{ t: 60, r: 10 }},
        buttons: [
          {{
            label: 'Play',
            method: 'animate',
            args: [null, {{
              mode: 'immediate',
              fromcurrent: true,
              frame: {{ duration: 120, redraw: true }},
              transition: {{ duration: 0 }}
            }}]
          }},
          {{
            label: 'Pause',
            method: 'animate',
            args: [[null], {{
              mode: 'immediate',
              frame: {{ duration: 0, redraw: false }},
              transition: {{ duration: 0 }}
            }}]
          }}
        ]
      }}]
    }};
    Plotly.newPlot('plot', data, layout, {{responsive:true}}).then(() => {{
      Plotly.addFrames('plot', frames);
    }});
  </script>
</body>
</html>
"##,
        title = title,
        n_frames = anim.frames.len(),
        n_cables = n_cables,
        ax = serde_json::to_string(&ax).unwrap(),
        ay = serde_json::to_string(&ay).unwrap(),
        az = serde_json::to_string(&az).unwrap(),
        labels = serde_json::to_string(&labels).unwrap(),
        dx = first.dolly[0],
        dy = first.dolly[1],
        dz = first.dolly[2],
        init_cables = init_cables,
        workspace_trace = workspace_trace,
        frames_js = frames_js,
        slider_steps = slider_steps,
    );

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, html)
}

/// Convenience: scene at a translation-only pose.
pub fn write_scene_at(
    robot: &Robot,
    position: Vec3,
    path: &Path,
    title: &str,
) -> spyder_core::Result<()> {
    let scene = SceneSnapshot::from_robot(robot, &Pose::from_position(position))?;
    write_scene_html(&scene, path, title).map_err(|e| {
        spyder_core::SpyderError::Config(format!("write scene: {e}"))
    })?;
    Ok(())
}

/// Convenience: animated line trajectory (+ optional workspace overlay).
pub fn write_scene_line(
    robot: &Robot,
    start: Vec3,
    end: Vec3,
    segments: usize,
    path: &Path,
    title: &str,
    workspace: Option<&WorkspaceReport>,
) -> spyder_core::Result<()> {
    let mut anim = SceneAnimation::from_line(robot, start, end, segments)?;
    if let Some(ws) = workspace {
        anim = anim.with_workspace(ws);
    }
    write_scene_animation_html(&anim, path, title).map_err(|e| {
        spyder_core::SpyderError::Config(format!("write scene animation: {e}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use crate::{sample_wrench_feasible, SampleBox};
    use nalgebra::DVector;
    use spyder_core::Preset;
    use std::path::PathBuf;

    #[test]
    fn writes_scene_html() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-scene");
        write_scene_at(
            &robot,
            Vec3::new(0.2, -0.1, 1.0),
            &dir.join("scene.html"),
            "test",
        )
        .unwrap();
        assert!(dir.join("scene.html").exists());
    }

    #[test]
    fn writes_animated_scene_with_workspace() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let box_ = SampleBox {
            min: Vec3::new(-1.0, -1.0, 0.5),
            max: Vec3::new(1.0, 1.0, 2.0),
            nx: 4,
            ny: 4,
            nz: 3,
        };
        let w = DVector::from_vec(vec![0.0, 0.0, -9.81]);
        let report = sample_wrench_feasible(&robot, &box_, w, 0.5, 500.0);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-scene");
        write_scene_line(
            &robot,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.4, 0.0, 1.2),
            6,
            &dir.join("anim.html"),
            "anim test",
            Some(&report),
        )
        .unwrap();
        let html = std::fs::read_to_string(dir.join("anim.html")).unwrap();
        assert!(html.contains("Plotly.addFrames"));
        assert!(html.contains("Play"));
        assert!(html.contains("workspace"));
    }

    #[test]
    fn scene_snapshot_matches_ik_lengths() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let pose = Pose::from_position(Vec3::new(0.1, -0.1, 1.2));
        let snap = SceneSnapshot::from_robot(&robot, &pose).unwrap();
        let ik = robot.ik(&pose).unwrap();
        assert_eq!(snap.lengths, ik.lengths);
        assert_relative_eq!(snap.dolly[2], 1.2);
        assert_eq!(snap.anchors.len(), 4);
    }

    #[test]
    fn scene_snapshot_has_cable_paths() {
        let mut robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        robot.cable_model = spyder_core::CableModelKind::Pulley {
            default_radius: 0.06,
        };
        for a in &mut robot.anchors {
            a.pulley_axis = Some(Vec3::z());
            a.pulley_radius = 0.06;
        }
        let pose = Pose::from_position(Vec3::new(0.2, -0.1, 1.0));
        let snap = SceneSnapshot::from_robot(&robot, &pose).unwrap();
        assert_eq!(snap.cable_paths.len(), 4);
        assert!(snap.cable_paths[0].len() >= 3);
        assert_eq!(snap.model, "pulley");
    }
}
