//! Ideal straight, massless, inextensible cable: Euclidean distance.

use crate::model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};

/// Ideal cable model \(L = \|B - A\|\).
#[derive(Clone, Copy, Debug, Default)]
pub struct Ideal;

impl CableModel for Ideal {
    fn length(&self, a: &Vec3, b: &Vec3, _ctx: &CableContext) -> CableResult<CableLength> {
        let d = (b - a).norm();
        if d <= f64::EPSILON {
            return Err(CableModelError::Geometry(
                "zero-length cable".into(),
            ));
        }
        Ok(CableLength {
            geometric: d,
            unstrained: None,
        })
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
        let m = Ideal;
        let len = m
            .length(&a, &b, &CableContext::default())
            .expect("length");
        assert_relative_eq!(len.geometric, 5.0);
    }

    #[test]
    fn zero_length_returns_geometry_error() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let m = Ideal;
        assert!(m.length(&a, &a, &CableContext::default()).is_err());
    }
}
