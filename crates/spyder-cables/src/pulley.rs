//! Swivel-pulley cable length: free-span tangent + wrap arc.

use crate::geometry::CableGeometry;
use crate::model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};
use crate::pulley_geom::{pulley_geometry, PulleyGeomInput};

/// Swivel pulley with fixed axis and radius at the base exit.
#[derive(Clone, Debug)]
pub struct Pulley {
    /// Unit axis of the swivel pulley (world frame).
    pub axis: Vec3,
    /// Pulley radius in meters.
    pub radius: f64,
    /// Optional unit direction the cable leaves the rim toward the winch (⊥ axis).
    pub winch_exit: Option<Vec3>,
    /// Constant rim-to-encoder segment (meters).
    pub runout_m: f64,
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
            winch_exit: None,
            runout_m: 0.0,
        })
    }

    /// Set winch exit azimuth (unit vector in world frame, need not be ⊥ axis).
    pub fn with_winch_exit(mut self, winch_exit: Vec3) -> Self {
        self.winch_exit = Some(winch_exit);
        self
    }

    /// Set constant rim-to-encoder runout.
    pub fn with_runout(mut self, runout_m: f64) -> Self {
        self.runout_m = runout_m.max(0.0);
        self
    }

    fn input(&self, center: &Vec3) -> PulleyGeomInput {
        PulleyGeomInput {
            center: *center,
            axis: self.axis,
            radius: self.radius,
            winch_exit: self.winch_exit,
            runout_m: self.runout_m,
        }
    }
}

impl CableModel for Pulley {
    fn geometry(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> CableResult<CableGeometry> {
        pulley_geometry(b, &self.input(a))
    }

    fn length(&self, a: &Vec3, b: &Vec3, ctx: &CableContext) -> CableResult<CableLength> {
        let g = self.geometry(a, b, ctx)?;
        Ok(CableLength {
            geometric: g.geometric,
            unstrained: g.unstrained,
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
            .geometry(&a, &b, &CableContext::default())
            .unwrap()
            .geometric;
        let pulley = Pulley::new(Vec3::z(), 0.05).unwrap();
        let plen = pulley
            .geometry(&a, &b, &CableContext::default())
            .unwrap()
            .geometric;
        assert!(plen >= ideal * 0.99, "pulley {plen} vs ideal {ideal}");
        assert_relative_eq!(ideal, 2.0);
    }

    #[test]
    fn pulley_pull_along_tangent_not_chord() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.5);
        let pulley = Pulley::new(Vec3::z(), 0.1).unwrap();
        let g = pulley.geometry(&a, &b, &CableContext::default()).unwrap();
        let chord = (a - b).normalize();
        assert!((g.unit_pull - chord).norm() > 1e-4);
    }

    #[test]
    fn zero_radius_matches_ideal() {
        let a = Vec3::new(0.0, 0.0, 1.0);
        let b = Vec3::new(1.0, 2.0, 0.0);
        let pulley = Pulley::new(Vec3::z(), 0.0).unwrap();
        let p = pulley.geometry(&a, &b, &CableContext::default()).unwrap();
        let i = Ideal.geometry(&a, &b, &CableContext::default()).unwrap();
        assert_relative_eq!(p.geometric, i.geometric);
        assert_relative_eq!(p.unit_pull.x, i.unit_pull.x, epsilon = 1e-9);
    }

    #[test]
    fn attachment_inside_pulley_cylinder_errors() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.01, 0.0, 0.0);
        let pulley = Pulley::new(Vec3::z(), 0.05).unwrap();
        assert!(pulley.geometry(&a, &b, &CableContext::default()).is_err());
    }

    #[test]
    fn zero_axis_errors() {
        assert!(Pulley::new(Vec3::zeros(), 0.05).is_err());
    }
}
