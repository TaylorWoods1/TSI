//! Sag cable geometry in the plane spanned by the chord and gravity.

use crate::geometry::{sag_plane_down, CableGeometry};
use crate::model::{CableModelError, CableResult, Vec3};
use crate::sag::Sag;

/// Gravity direction (unit, pointing down).
fn gravity_down(g: f64) -> Vec3 {
    Vec3::new(0.0, 0.0, -g.signum())
}

/// Horizontal span and vertical rise of chord `rel = b - a` for gravity along -Z.
pub fn sag_spans(rel: &Vec3) -> (f64, f64) {
    let l_h = Vec3::new(rel.x, rel.y, 0.0).norm();
    let dz = rel.z;
    (l_h, dz)
}

/// Full sag geometry at a known scalar tension.
pub fn sag_geometry(sag: &Sag, a: &Vec3, b: &Vec3, tension: f64) -> CableResult<CableGeometry> {
    if tension <= 0.0 {
        return Err(CableModelError::Context(
            "tension must be positive for sag".into(),
        ));
    }
    let rel = b - a;
    let l_geom = rel.norm();
    if l_geom <= f64::EPSILON {
        return Err(CableModelError::Geometry("zero-length cable".into()));
    }
    let (l_h, dz) = sag_spans(&rel);
    let (h, v) = sag.tension_components(&rel, l_geom, l_h, tension)?;
    let l0 = sag.irvine_unstrained(l_h, dz, h, v)?;

    let chord_hat = rel / l_geom;
    let g_down = gravity_down(sag.g);
    let toward_anchor = -chord_hat;
    let w = sag.mu * sag.g;
    let sag_slope = (w * l_h.max(1e-9)) / (2.0 * tension);
    let mut pull = toward_anchor + g_down * sag_slope;
    if let Some(down_in_plane) = sag_plane_down(&chord_hat, &g_down) {
        pull += down_in_plane * sag_slope * 0.5;
    }
    let pn = pull.norm();
    let unit_pull = if pn > f64::EPSILON {
        pull / pn
    } else {
        toward_anchor
    };

    Ok(CableGeometry {
        geometric: l_geom,
        unstrained: Some(l0),
        unit_pull,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn sag_pull_deviates_from_chord_under_low_tension() {
        let sag = Sag {
            mu: 1.0,
            ea: 1.0e6,
            g: 9.81,
        };
        let a = Vec3::new(0.0, 0.0, 5.0);
        let b = Vec3::new(10.0, 0.0, 0.0);
        let g_hi = sag_geometry(&sag, &a, &b, 500.0).unwrap();
        let g_lo = sag_geometry(&sag, &a, &b, 50.0).unwrap();
        let chord = (a - b).normalize();
        let d_hi = (g_hi.unit_pull - chord).norm();
        let d_lo = (g_lo.unit_pull - chord).norm();
        assert!(d_lo >= d_hi);
    }
}
