//! Ideal massless straight cable.

use crate::geometry::CableGeometry;
use crate::model::{CableContext, CableLength, CableModel, CableResult, Vec3};

/// Inextensible straight cable.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ideal;

impl CableModel for Ideal {
    fn length(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> CableResult<CableLength> {
        let g = self.geometry(a, b, &CableContext::default())?;
        Ok(CableLength {
            geometric: g.geometric,
            unstrained: g.unstrained,
        })
    }

    fn geometry(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> CableResult<CableGeometry> {
        CableGeometry::ideal(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn ideal_length_is_euclidean() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(3.0, 4.0, 0.0);
        let g = Ideal.geometry(&a, &b, &CableContext::default()).unwrap();
        assert_relative_eq!(g.geometric, 5.0);
        assert_relative_eq!(g.unit_pull.x, -0.6);
        assert_relative_eq!(g.unit_pull.y, -0.8);
    }

    #[test]
    fn zero_length_returns_geometry_error() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        assert!(Ideal.length(&a, &a, &CableContext::default()).is_err());
    }
}
