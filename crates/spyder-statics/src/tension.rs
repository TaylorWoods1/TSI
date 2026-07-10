//! Cable tension distribution (closed-form + bounds check).

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

/// Feasibility wrapper.
pub fn is_wrench_feasible(
    a: &DMatrix<f64>,
    wrench: &DVector<f64>,
    f_min: f64,
    f_max: f64,
) -> bool {
    closed_form_tensions(a, wrench, f_min, f_max).is_ok()
}
