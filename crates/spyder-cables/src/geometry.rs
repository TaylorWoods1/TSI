//! Shared cable geometry: length and unit pull direction at the platform attachment.

use crate::model::{CableModelError, CableResult, Vec3};

/// Full geometric evaluation of one cable at an attachment point.
#[derive(Clone, Debug, PartialEq)]
pub struct CableGeometry {
    /// Measured / commanded path length (meters).
    pub geometric: f64,
    /// Unstrained rest length when distinguished (sag).
    pub unstrained: Option<f64>,
    /// Unit vector at attachment `B`: direction the cable pulls on the platform
    /// (from attachment toward the cable / anchor along the free span).
    pub unit_pull: Vec3,
}

impl CableGeometry {
    /// Ideal straight cable between anchor `a` and attachment `b`.
    pub fn ideal(a: &Vec3, b: &Vec3) -> CableResult<Self> {
        let rel = b - a;
        let d = rel.norm();
        if d <= f64::EPSILON {
            return Err(CableModelError::Geometry("zero-length cable".into()));
        }
        Ok(Self {
            geometric: d,
            unstrained: None,
            unit_pull: -rel / d,
        })
    }
}

/// Foot of the perpendicular from `b` onto the axis through `center` with direction `axis`.
pub fn axis_foot(center: &Vec3, axis: &Vec3, b: &Vec3) -> (Vec3, Vec3, f64) {
    let axial = (b - center).dot(axis);
    let c = center + axis * axial;
    let radial = b - c;
    let rho = radial.norm();
    (c, radial, rho)
}

/// Unit vector in the sag plane perpendicular to chord `chord_hat`, aligned with gravity `g`.
pub fn sag_plane_down(chord_hat: &Vec3, g: &Vec3) -> Option<Vec3> {
    let g_tangent = g - chord_hat * g.dot(chord_hat);
    let n = g_tangent.norm();
    if n <= f64::EPSILON {
        None
    } else {
        Some(g_tangent / n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn ideal_unit_pull_points_attachment_toward_anchor() {
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let g = CableGeometry::ideal(&a, &b).unwrap();
        assert_relative_eq!(g.geometric, (5.0f64).hypot(1.0), epsilon = 1e-9);
        let u = g.unit_pull;
        assert_relative_eq!(u.x, -1.0 / g.geometric, epsilon = 1e-9);
        assert_relative_eq!(u.z, 5.0 / g.geometric, epsilon = 1e-9);
    }
}
