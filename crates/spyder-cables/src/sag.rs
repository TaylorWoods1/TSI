//! Irvine elastic catenary (static sag) cable model.
//!
//! Given horizontal force component `h` and vertical force component `v` at the
//! attachment (or recovered from total tension and chord geometry), the
//! unstrained length follows Irvine's equations. Phase 1 provides a practical
//! solver: from chord geometry + scalar tension magnitude, estimate `h`/`v`
//! from the chord direction and evaluate \(L_0\).

use crate::model::{CableContext, CableLength, CableModel, CableModelError, CableResult, Vec3};

/// Sagging cable with mass per unit length and axial stiffness.
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
    /// spans `dx`, `dz`.
    ///
    /// Uses the elastic catenary engineering approximation:
    /// \(L_0 \approx L_{\text{chord}} + \dfrac{w^2 l_h^3}{24 T^2} - \dfrac{TL_{\text{chord}}}{EA}\)
    /// where sag adds arc length over the chord and elasticity subtracts stretch.
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
        let w = self.mu * self.g; // weight per length (N/m)
        let l_geom = (dx_horizontal * dx_horizontal + dz * dz).sqrt();
        if l_geom <= f64::EPSILON {
            return Err(CableModelError::Geometry("zero chord".into()));
        }
        let t = (h * h + v * v).sqrt();
        // Parabolic sag excess: ΔL ≈ w² l_h³ / (24 T²)
        let l_h = dx_horizontal.max(1e-9);
        let sag_term = (w * w * l_h.powi(3)) / (24.0 * t * t);
        // Elastic stretch removal: L_strained ≈ L₀ + TL/EA  =>  L₀ ≈ L_strained − TL/EA
        let elastic = t * l_geom / self.ea;
        let l0 = l_geom + sag_term - elastic;
        if l0 <= 0.0 {
            return Err(CableModelError::Numeric(
                "computed unstrained length non-positive".into(),
            ));
        }
        Ok(l0)
    }

    /// Decompose scalar tension along the chord into catenary horizontal `h`
    /// and vertical `v` components, iterating once for elastic consistency.
    fn tension_components(
        &self,
        rel: &Vec3,
        l_geom: f64,
        dx: f64,
        tension: f64,
    ) -> CableResult<(f64, f64)> {
        if l_geom <= f64::EPSILON {
            return Err(CableModelError::Geometry("zero chord".into()));
        }
        // Project total tension onto chord-aligned horizontal/vertical axes.
        let mut h = if dx > 1e-6 {
            tension * (dx / l_geom)
        } else {
            1e-6
        };
        let mut v = tension * (rel.z / l_geom);

        // One refinement pass: re-estimate h from updated unstrained length.
        if let Ok(l0) = self.irvine_unstrained(dx, rel.z, h, v) {
            let elastic_ratio = tension / self.ea;
            let l_strained_est = l0 * (1.0 + elastic_ratio);
            if l_strained_est > f64::EPSILON {
                let scale = l_geom / l_strained_est;
                h = (h * scale).max(1e-6);
                v *= scale;
            }
        }
        Ok((h, v))
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
        let dx = Vec3::new(rel.x, rel.y, 0.0).norm();
        let (h, v) = self.tension_components(&rel, l_geom, dx, tension)?;
        let l0 = self.irvine_unstrained(dx, rel.z, h, v)?;
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
    fn sag_unstrained_less_than_geometric_under_high_tension() {
        // High tension: elastic shortening dominates sag excess.
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
        let l0 = len.unstrained.unwrap();
        assert!(
            l0 < len.geometric,
            "L0={l0} should be < geometric={}",
            len.geometric
        );
        assert_relative_eq!(len.geometric, 10.0f64.hypot(5.0), epsilon = 1e-9);
    }

    #[test]
    fn sag_unstrained_greater_than_geometric_under_low_tension() {
        // Low tension / heavy cable: sag excess dominates elasticity.
        let sag = Sag {
            mu: 2.0,
            ea: 1.0e6,
            g: 9.81,
        };
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(10.0, 0.0, 0.0);
        let ctx = CableContext {
            tension: Some(50.0),
        };
        let len = sag.length(&a, &b, &ctx).unwrap();
        let l0 = len.unstrained.unwrap();
        assert!(
            l0 > len.geometric,
            "L0={l0} should be > geometric={}",
            len.geometric
        );
    }

    #[test]
    fn missing_tension_errors() {
        let sag = Sag::default();
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(10.0, 0.0, 0.0);
        assert!(sag.length(&a, &b, &CableContext::default()).is_err());
    }

    #[test]
    fn non_positive_tension_errors() {
        let sag = Sag::default();
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(10.0, 0.0, 0.0);
        let ctx = CableContext { tension: Some(0.0) };
        assert!(sag.length(&a, &b, &ctx).is_err());
    }
}
