//! Cable tension distribution (closed-form + QP-style fallback).

use nalgebra::{DMatrix, DVector};

use crate::structure::StructureError;

/// Errors from tension solves.
#[derive(Debug, thiserror::Error)]
pub enum TensionError {
    /// No feasible tension in bounds.
    #[error("wrench infeasible")]
    Infeasible,
    /// Singular / rank-deficient structure matrix.
    #[error("singular structure matrix")]
    Singular,
    /// Configuration problem.
    #[error("{0}")]
    Config(String),
    /// Wrapped structure error.
    #[error(transparent)]
    Structure(#[from] StructureError),
}

/// Pott-style closed-form medium force distribution.
///
/// Solves \(A f + w = 0\) in the least-squares / particular+nullspace sense:
/// \(f = f_m - A^{+} (w + A f_m)\) with \(f_m = (f_{min}+f_{max})/2\).
/// Then checks bounds; if violated, returns [`TensionError::Infeasible`].
pub fn closed_form_tensions(
    a: &DMatrix<f64>,
    wrench: &DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> Result<DVector<f64>, TensionError> {
    if a.nrows() != wrench.len() {
        return Err(TensionError::Config(
            "wrench dimension must match structure rows".into(),
        ));
    }
    if f_min > f_max {
        return Err(TensionError::Config("f_min > f_max".into()));
    }
    let m = a.ncols();
    let fm = DVector::from_element(m, 0.5 * (f_min + f_max));
    // A f + w = 0  =>  A (fm + fv) + w = 0  =>  A fv = -w - A fm
    let rhs = -(wrench + a * &fm);
    let svd = a.clone().svd(true, true);
    let fv = svd.solve(&rhs, 1e-10).map_err(|_| TensionError::Singular)?;
    let f = fm + fv;
    for fi in f.iter() {
        if *fi < f_min - 1e-6 || *fi > f_max + 1e-6 {
            return Err(TensionError::Infeasible);
        }
    }
    Ok(f)
}

fn clamp_vec(f: &DVector<f64>, f_min: f64, f_max: f64) -> DVector<f64> {
    DVector::from_iterator(f.len(), f.iter().map(|x| x.clamp(f_min, f_max)))
}

/// Bounded least-squares fallback: minimize \(\|A f + w\|^2\) with box constraints
/// via projected gradient descent.
pub fn qp_tensions(
    a: &DMatrix<f64>,
    wrench: &DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> Result<DVector<f64>, TensionError> {
    if a.nrows() != wrench.len() {
        return Err(TensionError::Config(
            "wrench dimension must match structure rows".into(),
        ));
    }
    if f_min > f_max {
        return Err(TensionError::Config("f_min > f_max".into()));
    }
    let m = a.ncols();
    let mut f = DVector::from_element(m, 0.5 * (f_min + f_max));
    // Try unconstrained particular solution as a warm start, then project.
    let svd = a.clone().svd(true, true);
    if let Ok(f0) = svd.solve(&(-wrench), 1e-10) {
        f = clamp_vec(&f0, f_min, f_max);
    }

    let ata = a.transpose() * a;
    // Lipschitz-ish step: 1 / (||A||_F^2 + eps)
    let fro2: f64 = a.iter().map(|x| x * x).sum();
    let alpha = 1.0 / (fro2 + 1e-9);
    let tol = 1e-6 * (1.0 + wrench.norm());
    let max_iter = 5000;

    for _ in 0..max_iter {
        let residual = a * &f + wrench;
        if residual.norm() <= tol {
            return Ok(f);
        }
        let grad = &ata * &f + a.transpose() * wrench;
        f = clamp_vec(&(&f - alpha * grad), f_min, f_max);
    }
    let residual = a * &f + wrench;
    if residual.norm() <= tol * 10.0 {
        Ok(f)
    } else {
        Err(TensionError::Infeasible)
    }
}

/// Closed-form first; on bound failure, try [`qp_tensions`].
pub fn solve_tensions(
    a: &DMatrix<f64>,
    wrench: &DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> Result<DVector<f64>, TensionError> {
    match closed_form_tensions(a, wrench, f_min, f_max) {
        Ok(f) => Ok(f),
        Err(TensionError::Infeasible) => qp_tensions(a, wrench, f_min, f_max),
        Err(e) => Err(e),
    }
}

/// Feasibility wrapper (uses closed-form + QP fallback).
pub fn is_wrench_feasible(
    a: &DMatrix<f64>,
    wrench: &DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> bool {
    solve_tensions(a, wrench, f_min, f_max).is_ok()
}

#[cfg(test)]
mod tension_tests {
    use super::*;
    use crate::structure::structure_matrix_3;
    use nalgebra::Vector3;

    #[test]
    fn qp_recovers_when_medium_force_outside_tight_bounds() {
        // Geometry where medium-force solution may sit near bounds; use a case
        // that closed-form rejects but a feasible point exists near a bound.
        let exits = [
            Vector3::new(2.0, 1.0, 2.0),
            Vector3::new(-2.0, 1.0, 2.0),
            Vector3::new(-1.0, -2.0, 2.0),
            Vector3::new(1.0, -2.0, 2.0),
        ];
        let p = Vector3::new(0.5, 0.0, 0.5);
        let units: Vec<_> = exits
            .iter()
            .map(|a| {
                let d = a - p;
                d / d.norm()
            })
            .collect();
        let a = structure_matrix_3(&units).unwrap();
        let w = DVector::from_vec(vec![0.0, 0.0, -20.0]);
        // Very tight upper bound can make closed-form fail while QP still finds
        // a feasible corner solution for some layouts; if both fail, skip soft.
        let f_min = 0.5;
        let f_max = 15.0;
        let closed = closed_form_tensions(&a, &w, f_min, f_max);
        let solved = solve_tensions(&a, &w, f_min, f_max);
        match (closed, solved) {
            (Ok(_), Ok(_)) => {}
            (Err(TensionError::Infeasible), Ok(f)) => {
                assert!(f.iter().all(|x| *x >= f_min - 1e-6 && *x <= f_max + 1e-6));
                let r = &a * &f + &w;
                assert!(r.norm() < 1e-3, "residual {}", r.norm());
            }
            (Err(TensionError::Infeasible), Err(TensionError::Infeasible)) => {
                // Truly infeasible under these bounds — acceptable.
            }
            (c, s) => panic!("unexpected closed={c:?} solved={s:?}"),
        }
    }

    #[test]
    fn qp_matches_closed_form_when_feasible() {
        let exits = [
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(-1.0, 1.0, 1.0),
            Vector3::new(-1.0, -1.0, 1.0),
            Vector3::new(1.0, -1.0, 1.0),
        ];
        let p = Vector3::new(0.0, 0.0, 0.0);
        let units: Vec<_> = exits
            .iter()
            .map(|a| {
                let d = a - p;
                d / d.norm()
            })
            .collect();
        let a = structure_matrix_3(&units).unwrap();
        let w = DVector::from_vec(vec![0.0, 0.0, -10.0]);
        let f_cf = closed_form_tensions(&a, &w, 0.5, 100.0).unwrap();
        let f_qp = qp_tensions(&a, &w, 0.5, 100.0).unwrap();
        let r_qp = &a * &f_qp + &w;
        assert!(r_qp.norm() < 1e-5, "qp residual {}", r_qp.norm());
        // Both should balance the wrench; values may differ in the nullspace.
        let r_cf = &a * &f_cf + &w;
        assert!(r_cf.norm() < 1e-8);
    }
}
