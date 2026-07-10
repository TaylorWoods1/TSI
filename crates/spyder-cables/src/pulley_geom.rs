//! Swivel-pulley geometry: tangent span, rim wrap, and pull direction.

use crate::geometry::CableGeometry;
use crate::model::{CableModelError, CableResult, Vec3};

/// Inputs for pulley path geometry at a swivel cylinder.
#[derive(Clone, Debug)]
pub struct PulleyGeomInput {
    /// Pulley center on the base (world frame).
    pub center: Vec3,
    /// Unit swivel axis (world frame).
    pub axis: Vec3,
    /// Rim radius (meters).
    pub radius: f64,
    /// Optional unit direction the cable leaves the rim toward the winch ( ⊥ axis ).
    pub winch_exit: Option<Vec3>,
    /// Constant rim-to-encoder segment along the winch run (meters).
    pub runout_m: f64,
}

/// Evaluate pulley-compensated length and attachment pull direction.
pub fn pulley_geometry(b: &Vec3, input: &PulleyGeomInput) -> CableResult<CableGeometry> {
    if input.radius <= f64::EPSILON {
        return CableGeometry::ideal(&input.center, b);
    }
    let axis = input.axis;
    let r = input.radius;
    let c = input.center;
    let rel = b - c;
    let axial = rel.dot(&axis);
    let radial_vec = rel - axis * axial;
    let rho = radial_vec.norm();
    if rho <= r + 1e-12 {
        return Err(CableModelError::Geometry(
            "attachment inside pulley cylinder; no real tangent".into(),
        ));
    }

    let cb = radial_vec / rho;
    let alpha = (r / rho).clamp(-1.0, 1.0).acos();
    let sin_a = (1.0 - (r / rho).powi(2)).max(0.0).sqrt();
    let perp = axis.cross(&cb);
    let perp_n = perp.norm();
    let perp_hat = if perp_n > f64::EPSILON {
        perp / perp_n
    } else {
        return Err(CableModelError::Geometry(
            "attachment on pulley axis; indeterminate tangent".into(),
        ));
    };

    // External tangent point on the rim in the plane through `c`, `b`, and `axis`.
    let t = c + r * (cb * alpha.cos() + perp_hat * sin_a);
    let free = (b - t).norm();
    if free <= f64::EPSILON {
        return Err(CableModelError::Geometry("degenerate pulley tangent".into()));
    }
    let unit_pull = (t - b) / free;

    let mut arc = 0.0;
    if let Some(winch) = input.winch_exit {
        let mut w = winch - axis * winch.dot(&axis);
        let wn = w.norm();
        if wn > f64::EPSILON {
            w /= wn;
            let t_dir = (t - c).normalize();
            let dot = t_dir.dot(&w).clamp(-1.0, 1.0);
            let cross = t_dir.cross(&w);
            let signed = cross.dot(&axis).signum();
            arc = r * dot.acos() * signed.abs();
        }
    } else {
        // Minimal wrap from tangent point to the radial reference on the rim.
        arc = r * alpha;
    }

    let length = free + arc + input.runout_m.max(0.0);
    Ok(CableGeometry {
        geometric: length,
        unstrained: None,
        unit_pull,
    })
}

/// Polyline vertices for rendering: rim arc (when present) → tangent → attachment.
pub fn pulley_visual_polyline(
    b: &Vec3,
    input: &PulleyGeomInput,
    arc_segments: usize,
) -> CableResult<Vec<Vec3>> {
    if input.radius <= f64::EPSILON {
        return Ok(vec![input.center, *b]);
    }
    let axis = input.axis;
    let r = input.radius;
    let c = input.center;
    let rel = b - c;
    let axial = rel.dot(&axis);
    let radial_vec = rel - axis * axial;
    let rho = radial_vec.norm();
    if rho <= r + 1e-12 {
        return Err(CableModelError::Geometry(
            "attachment inside pulley cylinder; no real tangent".into(),
        ));
    }

    let cb = radial_vec / rho;
    let alpha = (r / rho).clamp(-1.0, 1.0).acos();
    let sin_a = (1.0 - (r / rho).powi(2)).max(0.0).sqrt();
    let perp = axis.cross(&cb);
    let perp_n = perp.norm();
    let perp_hat = if perp_n > f64::EPSILON {
        perp / perp_n
    } else {
        return Err(CableModelError::Geometry(
            "attachment on pulley axis; indeterminate tangent".into(),
        ));
    };

    let t = c + r * (cb * alpha.cos() + perp_hat * sin_a);
    let t_dir = (t - c) / r;

    let mut pts = Vec::new();
    let n = arc_segments.max(2);

    if let Some(winch) = input.winch_exit {
        let mut w = winch - axis * winch.dot(&axis);
        let wn = w.norm();
        if wn > f64::EPSILON {
            w /= wn;
            for i in 0..=n {
                let frac = i as f64 / n as f64;
                let dir = (w * (1.0 - frac) + t_dir * frac).normalize();
                pts.push(c + dir * r);
            }
        } else {
            pts.push(t);
        }
    } else {
        let ref_dir = cb;
        for i in 0..=n {
            let frac = i as f64 / n as f64;
            let angle = alpha * frac;
            let dir = (ref_dir * angle.cos() + perp_hat * angle.sin()).normalize();
            pts.push(c + dir * r);
        }
    }

    if pts.last().map(|p| (*p - t).norm()).unwrap_or(1.0) > 1e-9 {
        pts.push(t);
    }
    pts.push(*b);
    Ok(pts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn fixed_center_tangent_includes_vertical_span() {
        let input = PulleyGeomInput {
            center: Vec3::new(0.0, 0.0, 5.0),
            axis: Vec3::z(),
            radius: 0.05,
            winch_exit: None,
            runout_m: 0.0,
        };
        let b_low = Vec3::new(2.0, 0.0, 1.0);
        let b_high = Vec3::new(2.0, 0.0, 2.0);
        let g_low = pulley_geometry(&b_low, &input).unwrap();
        let g_high = pulley_geometry(&b_high, &input).unwrap();
        assert!(g_high.geometric < g_low.geometric);
    }

    #[test]
    fn horizontal_offset_uses_perpendicular_radius() {
        let input = PulleyGeomInput {
            center: Vec3::zeros(),
            axis: Vec3::z(),
            radius: 0.05,
            winch_exit: None,
            runout_m: 0.0,
        };
        let b = Vec3::new(2.0, 0.0, 1.0);
        let g = pulley_geometry(&b, &input).unwrap();
        let rho = 2.0f64;
        let expected_free = ((rho * rho - 0.05 * 0.05) + 1.0).sqrt();
        let alpha = (0.05 / rho).acos();
        let expected = expected_free + 0.05 * alpha;
        assert_relative_eq!(g.geometric, expected, epsilon = 1e-9);
    }

    #[test]
    fn winch_exit_adds_wrap_arc() {
        let input = PulleyGeomInput {
            center: Vec3::zeros(),
            axis: Vec3::z(),
            radius: 0.1,
            winch_exit: Some(Vec3::x()),
            runout_m: 0.0,
        };
        let b = Vec3::new(3.0, 0.0, 0.0);
        let g0 = pulley_geometry(&b, &input).unwrap();
        let mut no_winch = input.clone();
        no_winch.winch_exit = None;
        let g1 = pulley_geometry(&b, &no_winch).unwrap();
        assert!(g0.geometric >= g1.geometric);
    }
}
