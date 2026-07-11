//! Export workspace reports to CSV / JSON / HTML.

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::WorkspaceReport;

/// Write CSV with columns x,y,z,feasible.
pub fn write_csv(report: &WorkspaceReport, path: &Path) -> std::io::Result<()> {
    let mut f = fs::File::create(path)?;
    writeln!(f, "x,y,z,feasible")?;
    for s in &report.samples {
        writeln!(
            f,
            "{:.6},{:.6},{:.6},{}",
            s.x,
            s.y,
            s.z,
            if s.feasible { 1 } else { 0 }
        )?;
    }
    Ok(())
}

/// Write JSON report (pretty).
pub fn write_json(report: &WorkspaceReport, path: &Path) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(report)
        .map_err(std::io::Error::other)?;
    fs::write(path, text)
}

/// Write a self-contained HTML scatter plot (Plotly CDN) of feasible vs infeasible points.
pub fn write_html(report: &WorkspaceReport, path: &Path, title: &str) -> std::io::Result<()> {
    let mut fx = Vec::new();
    let mut fy = Vec::new();
    let mut fz = Vec::new();
    let mut ix = Vec::new();
    let mut iy = Vec::new();
    let mut iz = Vec::new();
    for s in &report.samples {
        if s.feasible {
            fx.push(s.x);
            fy.push(s.y);
            fz.push(s.z);
        } else {
            ix.push(s.x);
            iy.push(s.y);
            iz.push(s.z);
        }
    }
    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8"/>
  <title>{title}</title>
  <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
  <style>
    body {{ margin: 0; font-family: ui-sans-serif, system-ui, sans-serif; background: #0f1419; color: #e7ecf3; }}
    header {{ padding: 1rem 1.5rem; border-bottom: 1px solid #243044; }}
    h1 {{ margin: 0; font-size: 1.1rem; font-weight: 600; }}
    .meta {{ opacity: 0.7; font-size: 0.85rem; margin-top: 0.25rem; }}
    #plot {{ width: 100vw; height: calc(100vh - 72px); }}
  </style>
</head>
<body>
  <header>
    <h1>{title}</h1>
    <div class="meta">feasible {feasible}/{total} ({pct:.1}%)</div>
  </header>
  <div id="plot"></div>
  <script>
    const feasible = {{
      x: {fx}, y: {fy}, z: {fz},
      mode: 'markers', type: 'scatter3d', name: 'feasible',
      marker: {{ size: 3, color: '#3dd68c' }}
    }};
    const infeasible = {{
      x: {ix}, y: {iy}, z: {iz},
      mode: 'markers', type: 'scatter3d', name: 'infeasible',
      marker: {{ size: 2, color: '#5b6b7c', opacity: 0.35 }}
    }};
    Plotly.newPlot('plot', [feasible, infeasible], {{
      paper_bgcolor: '#0f1419',
      plot_bgcolor: '#0f1419',
      font: {{ color: '#e7ecf3' }},
      scene: {{
        xaxis: {{ title: 'X (m)' }},
        yaxis: {{ title: 'Y (m)' }},
        zaxis: {{ title: 'Z (m)' }},
        aspectmode: 'data'
      }},
      margin: {{ l: 0, r: 0, t: 0, b: 0 }}
    }}, {{responsive: true}});
  </script>
</body>
</html>
"##,
        title = title,
        feasible = report.feasible,
        total = report.total,
        pct = report.fraction * 100.0,
        fx = serde_json::to_string(&fx).unwrap(),
        fy = serde_json::to_string(&fy).unwrap(),
        fz = serde_json::to_string(&fz).unwrap(),
        ix = serde_json::to_string(&ix).unwrap(),
        iy = serde_json::to_string(&iy).unwrap(),
        iz = serde_json::to_string(&iz).unwrap(),
    );
    fs::write(path, html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sample_wrench_feasible, SampleBox};
    use nalgebra::DVector;
    use spyder_core::{Preset, Robot, Vec3};
    use std::path::PathBuf;

    #[test]
    fn export_files_roundtrip() {
        let robot = Robot::from_preset(Preset::Rect {
            width: 4.0,
            depth: 4.0,
            height: 3.0,
        })
        .unwrap();
        let box_ = SampleBox {
            min: Vec3::new(-1.0, -1.0, 0.5),
            max: Vec3::new(1.0, 1.0, 2.0),
            nx: 3,
            ny: 3,
            nz: 2,
        };
        let report = sample_wrench_feasible(
            &robot,
            &box_,
            DVector::from_vec(vec![0.0, 0.0, -9.81]),
            0.5,
            500.0,
        );
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/test-export");
        fs::create_dir_all(&dir).unwrap();
        write_csv(&report, &dir.join("ws.csv")).unwrap();
        write_json(&report, &dir.join("ws.json")).unwrap();
        write_html(&report, &dir.join("ws.html"), "test").unwrap();
        assert!(dir.join("ws.csv").exists());
        assert!(dir.join("ws.html").exists());
    }
}
