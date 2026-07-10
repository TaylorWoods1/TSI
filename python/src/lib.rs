//! PyO3 bindings for spyder.

use nalgebra::DVector;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use spyder_core::{Pose, Preset, Robot, Vec3};
use spyder_sim::{line_waypoints, sample_wrench_feasible, SampleBox};

fn to_py<E: std::fmt::Display>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// Python-facing robot wrapper.
#[pyclass(name = "Robot")]
struct PyRobot {
    inner: Robot,
}

#[pymethods]
impl PyRobot {
    /// Build a rectangular point-mass robot.
    #[staticmethod]
    #[pyo3(signature = (width, depth, height))]
    fn rect(width: f64, depth: f64, height: f64) -> PyResult<Self> {
        let inner = Robot::from_preset(Preset::Rect {
            width,
            depth,
            height,
        })
        .map_err(to_py)?;
        Ok(Self { inner })
    }

    /// Build a regular-polygon point-mass robot.
    #[staticmethod]
    #[pyo3(signature = (n, radius, height))]
    fn polygon(n: usize, radius: f64, height: f64) -> PyResult<Self> {
        let inner = Robot::from_preset(Preset::RegularPolygon { n, radius, height }).map_err(to_py)?;
        Ok(Self { inner })
    }

    /// Inverse kinematics for a translation-only pose. Returns cable lengths (meters).
    fn ik(&self, x: f64, y: f64, z: f64) -> PyResult<Vec<f64>> {
        let pose = Pose::from_position(Vec3::new(x, y, z));
        let ik = self.inner.ik(&pose).map_err(to_py)?;
        Ok(ik.lengths)
    }

    /// Forward kinematics from lengths. Returns (x, y, z, residual, method_name).
    fn fk(
        &self,
        lengths: Vec<f64>,
        seed_x: f64,
        seed_y: f64,
        seed_z: f64,
    ) -> PyResult<(f64, f64, f64, f64, String)> {
        let fk = self
            .inner
            .fk(&lengths, Vec3::new(seed_x, seed_y, seed_z))
            .map_err(to_py)?;
        Ok((
            fk.position.x,
            fk.position.y,
            fk.position.z,
            fk.residual,
            format!("{:?}", fk.method),
        ))
    }

    /// Wrench-feasible workspace fraction under gravity `mg` (force = -mg on Z).
    #[pyo3(signature = (xmin, xmax, ymin, ymax, zmin, zmax, nx, ny, nz, mg=9.81, f_min=0.5, f_max=500.0))]
    fn workspace_fraction(
        &self,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        zmin: f64,
        zmax: f64,
        nx: usize,
        ny: usize,
        nz: usize,
        mg: f64,
        f_min: f64,
        f_max: f64,
    ) -> PyResult<f64> {
        let box_ = SampleBox {
            min: Vec3::new(xmin, ymin, zmin),
            max: Vec3::new(xmax, ymax, zmax),
            nx,
            ny,
            nz,
        };
        let w = DVector::from_vec(vec![0.0, 0.0, -mg]);
        let report = sample_wrench_feasible(&self.inner, &box_, w, f_min, f_max);
        Ok(report.fraction)
    }

    /// Lengths along a straight line from start to end.
    fn line_ik(
        &self,
        x0: f64,
        y0: f64,
        z0: f64,
        x1: f64,
        y1: f64,
        z1: f64,
        segments: usize,
    ) -> PyResult<Vec<Vec<f64>>> {
        let pts = line_waypoints(
            Vec3::new(x0, y0, z0),
            Vec3::new(x1, y1, z1),
            segments,
        );
        let mut out = Vec::new();
        for p in pts {
            let ik = self.inner.ik(&Pose::from_position(p)).map_err(to_py)?;
            out.push(ik.lengths);
        }
        Ok(out)
    }
}

/// Spyder Python module.
#[pymodule]
fn spyder(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyRobot>()?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
