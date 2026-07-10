//! Layout presets that expand into anchor lists.

use crate::anchor::Anchor;
use crate::error::{Result, SpyderError};
use crate::types::Vec3;

/// Axis-aligned rectangle of `n=4` anchors centered on the origin in XY.
///
/// Corners at `(±width/2, ±depth/2, height)`.
pub fn rect(width: f64, depth: f64, height: f64) -> Result<Vec<Anchor>> {
    if width <= 0.0 || depth <= 0.0 {
        return Err(SpyderError::Config(
            "width and depth must be > 0".into(),
        ));
    }
    let hw = width / 2.0;
    let hd = depth / 2.0;
    let corners = [
        Vec3::new(hw, hd, height),
        Vec3::new(-hw, hd, height),
        Vec3::new(-hw, -hd, height),
        Vec3::new(hw, -hd, height),
    ];
    Ok(corners.into_iter().map(Anchor::point).collect())
}

/// Regular `n`-gon of anchors in a horizontal plane at `height`.
///
/// Vertices lie on a circle of `radius` centered on the Z axis. Vertex 0 is at
/// angle 0 (positive X).
pub fn regular_polygon(n: usize, radius: f64, height: f64) -> Result<Vec<Anchor>> {
    if n < 3 {
        return Err(SpyderError::Config("n must be >= 3".into()));
    }
    if radius <= 0.0 {
        return Err(SpyderError::Config("radius must be > 0".into()));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let th = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        out.push(Anchor::point(Vec3::new(
            radius * th.cos(),
            radius * th.sin(),
            height,
        )));
    }
    Ok(out)
}

/// Equilateral triangle convenience wrapper around [`regular_polygon`].
pub fn triangle(radius: f64, height: f64) -> Result<Vec<Anchor>> {
    regular_polygon(3, radius, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn rect_preset_four_corners_at_height() {
        let anchors = rect(10.0, 6.0, 8.0).expect("ok");
        assert_eq!(anchors.len(), 4);
        for a in &anchors {
            assert_relative_eq!(a.exit.z, 8.0);
        }
        let xs: Vec<f64> = anchors.iter().map(|a| a.exit.x).collect();
        assert!(xs.iter().any(|x| (*x - 5.0).abs() < 1e-9));
        assert!(xs.iter().any(|x| (*x + 5.0).abs() < 1e-9));
    }

    #[test]
    fn regular_polygon_n5() {
        let anchors = regular_polygon(5, 4.0, 7.0).expect("ok");
        assert_eq!(anchors.len(), 5);
        for a in &anchors {
            let r = (a.exit.x * a.exit.x + a.exit.y * a.exit.y).sqrt();
            assert_relative_eq!(r, 4.0, epsilon = 1e-9);
            assert_relative_eq!(a.exit.z, 7.0);
        }
    }

    #[test]
    fn n_less_than_3_errors() {
        assert!(regular_polygon(2, 1.0, 1.0).is_err());
    }
}
