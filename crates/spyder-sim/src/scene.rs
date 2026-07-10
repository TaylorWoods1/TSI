//! 3D scene export: anchors, cables, dolly.

use std::fs;
use std::path::Path;

use serde::Serialize;
use spyder_core::{Pose, Robot, Vec3};

/// Snapshot of the robot at a pose for visualization.
#[derive(Clone, Debug, Serialize)]
pub struct SceneSnapshot {
    /// Anchor world positions.
    pub anchors: Vec<[f64; 3]>,
    /// Dolly / EE position.
    pub dolly: [f64; 3],
    /// Cable lengths.
    pub lengths: Vec<f64>,
    /// Attachment world points (same as dolly in point-mass mode).
    pub attachments: Vec<[f64; 3]>,
}

impl SceneSnapshot {
    /// Build from robot + pose via IK.
    pub fn from_robot(robot: &Robot, pose: &Pose) -> spyder_core::Result<Self> {
        let ik = robot.ik(pose)?;
        let anchors: Vec<[f64; 3]> = robot
            .anchors
            .iter()
            .map(|a| [a.exit.x, a.exit.y, a.exit.z])
            .collect();
        let attachments: Vec<[f64; 3]> = if robot.point_mass {
            vec![
                [pose.position.x, pose.position.y, pose.position.z];
                robot.anchors.len()
            ]
        } else {
            robot
                .attachments
                .iter()
                .map(|att| {
                    let p = pose.transform_point(&att.body_point);
                    [p.x, p.y, p.z]
                })
                .collect()
        };
        Ok(Self {
            anchors,
            dolly: [pose.position.x, pose.position.y, pose.position.z],
            lengths: ik.lengths,
            attachments,
        })
    }
}

/// Write interactive Plotly HTML with anchors, cables, and dolly.
pub fn write_scene_html(scene: &SceneSnapshot, path: &Path, title: &str) -> std::io::Result<()> {
    let mut cable_traces = String::new();
    for i in 0..scene.anchors.len() {
        let a = scene.anchors[i];
        let b = scene.attachments[i];
        cable_traces.push_str(&format!(
            r#"{{
      x: [{}, {}], y: [{}, {}], z: [{}, {}],
      mode: 'lines', type: 'scatter3d', name: 'cable {i}',
      line: {{ width: 6, color: '#f0a202' }},
      showlegend: false
    }},"#,
            a[0], b[0], a[1], b[1], a[2], b[2],
        ));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
