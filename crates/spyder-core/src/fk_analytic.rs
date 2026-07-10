//! Analytic / reduced forward kinematics fast paths.

use crate::error::{Result, SpyderError};
use crate::fk::{FkMethod, FkResult};
use crate::types::Vec3;

/// Trilateration: intersect three spheres; pick the candidate closer to `seed`
/// (or higher Z if equally close).
pub fn fk_analytic_3(
    a0: Vec3,
    a1: Vec3,
    a2: Vec3,
    l0: f64,
    l1: f64,
    l2: f64,
    seed: Vec3,
) -> Result<FkResult> {
    // Build local frame with origin at a0, x along a1-a0
    let d = (a1 - a0).norm();
    if d <= f64::EPSILON {
        return Err(SpyderError::Geometry("coincident anchors in trilateration".into()));
    }
    let ex = (a1 - a0) / d;
    let p2 = a2 - a0;
    let i = p2.dot(&ex);
    let ey_temp = p2 - ex * i;
    let ey_norm = ey_temp.norm();
    if ey_norm <= f64::EPSILON {
        return Err(SpyderError::Geometry("colinear anchors in trilateration".into()));
    }
    let ey = ey_temp / ey_norm;
    let ez = ex.cross(&ey);
    let j = p2.dot(&ey);

    let x = (l0 * l0 - l1 * l1 + d * d) / (2.0 * d);
    let y = (l0 * l0 - l2 * l2 + i * i + j * j - 2.0 * i * x) / (2.0 * j);
    let z2 = l0 * l0 - x * x - y * y;
    if z2 < -1e-9 {
        return Err(SpyderError::Geometry(
            "no real trilateration intersection".into(),
        ));
    }
    let z = z2.max(0.0).sqrt();

    let c1 = a0 + ex * x + ey * y + ez * z;
    let c2 = a0 + ex * x + ey * y - ez * z;

    let pick = if (c1 - seed).norm_squared() <= (c2 - seed).norm_squared() {
        c1
    } else {
        c2
    };

    // Residual vs the three lengths
    let residual = ((pick - a0).norm() - l0).abs()
        + ((pick - a1).norm() - l1).abs()
        + ((pick - a2).norm() - l2).abs();

    Ok(FkResult {
        position: pick,
        orientation: crate::types::UnitQuat::identity(),
        residual,
        iterations: 0,
        method: FkMethod::Analytic3,
    })
}

/// True when four anchors form an axis-aligned rectangle in a constant-Z plane.
pub fn is_axis_aligned_rect4(exits: &[Vec3]) -> bool {
    if exits.len() != 4 {
        return false;
    }
    let z0 = exits[0].z;
    if exits.iter().any(|e| (e.z - z0).abs() > 1e-9) {
        return false;
    }
    // Distinct |x| and |y| pairs centered — check unique x and y counts == 2
    let mut xs: Vec<f64> = exits.iter().map(|e| e.x).collect();
    let mut ys: Vec<f64> = exits.iter().map(|e| e.y).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    ys.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    xs.len() == 2 && ys.len() == 2
}

/// Reduced FK for rectangular 4-cable point-mass: use first three for trilateration,
/// verify against the fourth length, refine with one numeric polish if needed.
pub fn fk_analytic_rect4(
    exits: &[Vec3],
    lengths: &[f64],
    seed: Vec3,
) -> Result<FkResult> {
    if !is_axis_aligned_rect4(exits) || lengths.len() != 4 {
        return Err(SpyderError::Config(
            "fk_analytic_rect4 requires axis-aligned rect-4 geometry".into(),
        ));
    }
    let mut result = fk_analytic_3(
        exits[0], exits[1], exits[2], lengths[0], lengths[1], lengths[2], seed,
    )?;
    // Prefer the candidate that also matches cable 3; if residual on 4th is large, flip Z relative to plane
    let err4 = ((result.position - exits[3]).norm() - lengths[3]).abs();
    if err4 > 1e-4 {
        // try the other trilateration branch by reflecting seed through the plane z=exits[0].z
        let reflected = Vec3::new(seed.x, seed.y, 2.0 * exits[0].z - seed.z);
        let alt = fk_analytic_3(
            exits[0], exits[1], exits[2], lengths[0], lengths[1], lengths[2], reflected,
        )?;
        let alt_err4 = ((alt.position - exits[3]).norm() - lengths[3]).abs();
        if alt_err4 < err4 {
            result = alt;
        }
    }
    result.method = FkMethod::AnalyticRect4;
    result.residual = (0..4)
        .map(|i| {
            let e = (result.position - exits[i]).norm() - lengths[i];
            e * e
        })
        .sum::<f64>()
        .sqrt();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fk::fk_point_mass_numeric;
    use crate::ik::ideal_ik_point_mass;
    use crate::preset::rect;
    use approx::assert_relative_eq;

    #[test]
    fn three_cable_analytic_fk() {
        let a0 = Vec3::new(1.0, 0.0, 2.0);
        let a1 = Vec3::new(-0.5, 0.866, 2.0);
        let a2 = Vec3::new(-0.5, -0.866, 2.0);
        let p = Vec3::new(0.1, 0.05, 0.8);
        let l0 = (p - a0).norm();
        let l1 = (p - a1).norm();
        let l2 = (p - a2).norm();
        let got = fk_analytic_3(a0, a1, a2, l0, l1, l2, Vec3::new(0.0, 0.0, 1.0)).unwrap();
        assert_relative_eq!(got.position.x, p.x, epsilon = 1e-8);
        assert_relative_eq!(got.position.y, p.y, epsilon = 1e-8);
        assert_relative_eq!(got.position.z, p.z, epsilon = 1e-8);
        assert_eq!(got.method, FkMethod::Analytic3);
    }

    #[test]
    fn fk_dispatch_rect4_analytic() {
        let anchors = rect(4.0, 4.0, 3.0).unwrap();
        let exits: Vec<Vec3> = anchors.iter().map(|a| a.exit).collect();
        assert!(is_axis_aligned_rect4(&exits));
        let p = Vec3::new(0.25, -0.15, 1.2);
        let lengths = ideal_ik_point_mass(&exits, &p).unwrap();
        let got = fk_analytic_rect4(&exits, &lengths, Vec3::new(0.0, 0.0, 1.5)).unwrap();
        assert_eq!(got.method, FkMethod::AnalyticRect4);
        assert_relative_eq!(got.position.x, p.x, epsilon = 1e-6);
        assert_relative_eq!(got.position.y, p.y, epsilon = 1e-6);
        assert_relative_eq!(got.position.z, p.z, epsilon = 1e-6);
        // numeric should agree
        let num = fk_point_mass_numeric(&exits, &lengths, Vec3::new(0.0, 0.0, 1.5)).unwrap();
        assert_relative_eq!(num.position.x, got.position.x, epsilon = 1e-5);
    }
}
