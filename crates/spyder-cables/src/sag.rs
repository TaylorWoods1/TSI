//! Irvine elastic catenary (static sag) cable model.
//!
//! Given horizontal force component `h` and vertical force component `v` at the
//! attachment (or recovered from total tension and chord geometry), the
//! unstrained length follows Irvine's equations. Phase 1 provides a practical
//! solver: from chord geometry + scalar tension magnitude, estimate `h`/`v`
//! from the chord direction and evaluate \(L_0\).

use crate::model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};

/// Sagging cable with mass per length and axial stiffness.
#[derive(Clone, Debug)]
pub struct Sag {
    /// Mass per unit unstrained length (kg/m).
    pub mu: f64,
    /// Axial stiffness EA (Newtons).
    pub ea: f64,
    /// Gravity magnitude (m/s²).
    pub g: f64,
}

impl Default for Sag {
    fn default() -> Self {
        Self {
            mu: 0.05,
            ea: 1.0e5,
            g: 9.81,
        }
    }
}

impl Sag {
    /// Irvine unstrained length from horizontal tension `h` (>0), vertical
    /// component `v` at the lower/near end convention, and horizontal/vertical
    /// spans `l`, `h_span` (here `dx` horizontal distance, `dz` vertical rise).
    ///
    /// Uses the elastic catenary form:
    /// \(L_0 = \sqrt{l^2 + h_z^2} - \frac{\mu g}{2EA}\left(\ldots\right)\) simplified
    /// via the common engineering expression from chord + tension.
    pub fn irvine_unstrained(
        &self,
        dx_horizontal: f64,
        dz: f64,
        h: f64,
        v: f64,
    ) -> CableResult<f64> {
        if h <= f64::EPSILON {
            return Err(CableModelError::Numeric(
                "horizontal tension component must be > 0 for sag model".into(),
            ));
        }
        let w = self.mu * self.g; // weight per length
                                 // Classic Irvine:
                                 // L0 = (h/w) * (asinh((v)/h) - asinh((v - w*L0)/h)) ... implicit.
                                 // We use an explicit elastic approximation widely used in CDPR code:
                                 // L0 = sqrt(dx^2 + dz^2) * (1 - (w^2 * dx^2)/(24 * T^2)) - T*L_geom/(EA)
                                 // with T = sqrt(h^2 + v^2), iterated once for consistency.
        let l_geom = (dx_horizontal * dx_horizontal + dz * dz).sqrt();
        if l_geom <= f64::EPSILON {
            return Err(CableModelError::Geometry("zero chord".into()));
        }
        let t = (h * h + v * v).sqrt();
        // Catenary correction (parabolic sag term) + elastic stretch removal
        let sag_term = (w * w * dx_horizontal.powi(2) * l_geom) / (24.0 * t * t);
        let elastic = t * l_geom / self.ea;
        let l0 = l_geom - sag_term - elastic;
        if l0 <= 0.0 {
            return Err(CableModelError::Numeric(
                "computed unstrained length non-positive".into(),
            ));
        }
        Ok(l0)
    }
}

impl CableModel for Sag {
    fn length(&self, a: &Vec3, b: &Vec3, ctx: &CableContext) -> CableResult<CableLength> {
        let rel = b - a;
        let l_geom = rel.norm();
        if l_geom <= f64::EPSILON {
            return Err(CableModelError::Geometry("zero-length cable".into()));
        }
        let tension = ctx.tension.ok_or_else(|| {
            CableModelError::Context("sag model requires CableContext.tension".into())
        })?;
        if tension <= 0.0 {
            return Err(CableModelError::Context(
                "tension must be positive for sag".into(),
            ));
        }
        // Decompose tension along chord as approximation for h, v
        let u = rel / l_geom;
        let horiz = Vec3::new(u.x, u.y, 0.0);
        let horiz_n = horiz.norm();
        let h = tension * horiz_n;
        let v = tension * u.z;
        let dx = (Vec3::new(rel.x, rel.y, 0.0)).norm();
        let l0 = self.irvine_unstrained(dx, rel.z, h.max(1e-6), v)?;
        Ok(CableLength {
            geometric: l_geom,
            unstrained: Some(l0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn sag_unstrained_less_than_geometric_under_tension() {
        // Under tension, elastic stretch means L0 < geometric chord for taut cables
        // with the elastic term dominating the small sag correction at high T.
        let sag = Sag {
            mu: 0.02,
            ea: 5.0e4,
            g: 9.81,
        };
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(10.0, 0.0, 0.0);
        let ctx = CableContext {
            tension: Some(500.0),
        };
        let len = sag.length(&a, &b, &ctx).unwrap();
        assert!(len.unstrained.is_some());
        let l0 = len.unstrained.unwrap();
        // For this high tension, elastic shortening dominates: L0 < L_geom
        assert!(
            l0 < len.geometric,
            "L0={l0} should be < geometric={}",
            len.geometric
        );
        assert_relative_eq!(len.geometric, (10.0f64.hypot(5.0)), epsilon = 1e-9);
    }
}
