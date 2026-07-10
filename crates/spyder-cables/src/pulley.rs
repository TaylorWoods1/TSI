//! Swivel-pulley cable length: free-span tangent + wrap arc.

use crate::model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};

/// Swivel pulley with fixed axis and radius at the base exit.
///
/// The geometric length is the tangent segment from the platform attachment to
/// the pulley rim plus the arc wrapped on the pulley from a reference tangent
/// (cable leaving toward the winch along the plane perpendicular to the axis).
///
/// For Phase 1 we use a common practical model: length = free tangent length
/// from attachment to tangent point + `radius * alpha`, where `alpha` is the
/// wrap angle from the horizontal reference in the swivel plane.
#[derive(Clone, Debug)]
pub struct Pulley {
    /// Unit axis of the swivel pulley (world frame).
    pub axis: Vec3,
    /// Pulley radius in meters.
    pub radius: f64,
}

impl Pulley {
    /// Construct from axis and radius; axis is normalized.
    pub fn new(axis: Vec3, radius: f64) -> CableResult<Self> {
        if radius < 0.0 {
            return Err(CableModelError::Geometry(
                "pulley radius must be >= 0".into(),
            ));
        }
        let n = axis.norm();
        if n <= f64::EPSILON {
            return Err(CableModelError::Geometry(
                "pulley axis must be non-zero".into(),
            ));
        }
        Ok(Self {
            axis: axis / n,
            radius,
        })
    }
}

impl CableModel for Pulley {
    fn length(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> CableResult<CableLength> {
        if self.radius <= f64::EPSILON {
            let d = (b - a).norm();
            if d <= f64::EPSILON {
                return Err(CableModelError::Geometry("zero-length cable".into()));
            }
            return Ok(CableLength {
                geometric: d,
                unstrained: None,
            });
        }

        // Vector from pulley center (exit) to attachment.
        let rel = b - a;
        // Component along axis and in the swivel plane.
        let axial = rel.dot(&self.axis);
        let radial_vec = rel - self.axis * axial;
        let rho = radial_vec.norm();

        // Distance from attachment to pulley center projected such that a
        // tangent of length sqrt(rho^2 - r^2) exists in the plane, then
        // account for axial offset: free length = sqrt(tangent_planar^2 + axial^2)
        if rho <= self.radius + 1e-12 {
            return Err(CableModelError::Geometry(
                "attachment inside pulley cylinder; no real tangent".into(),
            ));
        }

        let planar_tangent = (rho * rho - self.radius * self.radius).sqrt();
        let free = (planar_tangent * planar_tangent + axial * axial).sqrt();

        // Wrap angle from the radial direction to the tangent touch point.
        // cos(alpha) = r/rho for the planar geometry; arc = r * acos(r/rho)
        // (minimal wrap from closest approach — standard first-order pulley term).
        let alpha = (self.radius / rho).clamp(-1.0, 1.0).acos();
        let arc = self.radius * alpha;

        Ok(CableLength {
            geometric: free + arc,
            unstrained: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ideal::Ideal;
    use approx::assert_relative_eq;

    #[test]
    fn pulley_length_exceeds_euclidean_when_radius_positive() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let ideal = Ideal
            .length(&a, &b, &CableContext::default())
            .unwrap()
            .geometric;
        let pulley = Pulley::new(Vec3::z(), 0.05).unwrap();
        let plen = pulley
            .length(&a, &b, &CableContext::default())
            .unwrap()
            .geometric;
        assert!(plen > ideal, "pulley {plen} should exceed ideal {ideal}");
        assert_relative_eq!(ideal, 2.0);
    }

    #[test]
    fn zero_radius_matches_ideal() {
        let a = Vec3::new(0.0, 0.0, 1.0);
        let b = Vec3::new(1.0, 2.0, 0.0);
        let pulley = Pulley::new(Vec3::z(), 0.0).unwrap();
        let p = pulley.length(&a, &b, &CableContext::default()).unwrap();
        let i = Ideal.length(&a, &b, &CableContext::default()).unwrap();
        assert_relative_eq!(p.geometric, i.geometric);
    }
}
