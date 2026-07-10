//! Structure (wrench) matrix assembly for CDPRs.

use nalgebra::{DMatrix, Vector3};

/// 3D vector alias.
pub type Vec3 = Vector3<f64>;

/// Build the 6×m structure matrix \(A^T\) for a spatial platform.
///
/// Column \(i\) is \([u_i; b_i \times u_i]\) where \(u_i\) is the unit vector
/// from platform attachment to base exit (cable pull direction on the platform),
/// and \(b_i\) is the world-frame attachment position relative to the platform
/// origin (moment arm). For point-mass, pass `moment_arms` as zeros → torque
/// rows are zero and only the force block is meaningful (use 3×m helper).
pub fn structure_matrix_6(
    unit_pulls: &[Vec3],
    moment_arms: &[Vec3],
) -> Result<DMatrix<f64>, StructureError> {
    if unit_pulls.len() != moment_arms.len() {
        return Err(StructureError::Config(
            "unit_pulls and moment_arms length mismatch".into(),
        ));
    }
    let m = unit_pulls.len();
    if m == 0 {
        return Err(StructureError::Config("need at least one cable".into()));
    }
    let mut a = DMatrix::zeros(6, m);
    for (j, (u, b)) in unit_pulls.iter().zip(moment_arms.iter()).enumerate() {
        let tau = b.cross(u);
        a[(0, j)] = u.x;
        a[(1, j)] = u.y;
        a[(2, j)] = u.z;
        a[(3, j)] = tau.x;
        a[(4, j)] = tau.y;
        a[(5, j)] = tau.z;
    }
    Ok(a)
}

/// 3×m structure matrix for point-mass (force only): columns are unit pull directions.
pub fn structure_matrix_3(unit_pulls: &[Vec3]) -> Result<DMatrix<f64>, StructureError> {
    let m = unit_pulls.len();
    if m == 0 {
        return Err(StructureError::Config("need at least one cable".into()));
    }
    let mut a = DMatrix::zeros(3, m);
    for (j, u) in unit_pulls.iter().enumerate() {
        a[(0, j)] = u.x;
        a[(1, j)] = u.y;
        a[(2, j)] = u.z;
    }
    Ok(a)
}

/// Structure assembly errors.
#[derive(Debug, thiserror::Error)]
pub enum StructureError {
    /// Bad inputs.
    #[error("invalid configuration: {0}")]
    Config(String),
}
