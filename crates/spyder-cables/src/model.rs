//! Shared cable model trait and length result types.

use crate::geometry::CableGeometry;

use nalgebra::Vector3;

/// 3D vector alias (meters) used by cable models.
pub type Vec3 = Vector3<f64>;

/// Extra context for models that need tension / material data.
#[derive(Clone, Debug, Default)]
pub struct CableContext {
    /// Optional tension at the cable (Newtons), used by sag models.
    pub tension: Option<f64>,
}

/// Length outputs from a cable model.
#[derive(Clone, Debug, PartialEq)]
pub struct CableLength {
    /// Geometric / chord (or tangent+arc) length in meters.
    pub geometric: f64,
    /// Unstrained rest length when the model distinguishes it (sag).
    pub unstrained: Option<f64>,
}

/// Error from a cable length evaluation.
#[derive(Debug, thiserror::Error)]
pub enum CableModelError {
    /// Degenerate geometry (e.g. zero length).
    #[error("{0}")]
    Geometry(String),
    /// Missing required context (e.g. tension for sag).
    #[error("{0}")]
    Context(String),
    /// Numerical failure inside the model.
    #[error("{0}")]
    Numeric(String),
}

/// Result of a cable model evaluation.
pub type CableResult<T> = std::result::Result<T, CableModelError>;

/// Trait implemented by ideal, pulley, and sag models.
pub trait CableModel {
    /// Compute cable length between base exit `a` and platform attachment `b` (world).
    fn length(&self, a: &Vec3, b: &Vec3, ctx: &CableContext) -> CableResult<CableLength>;

    /// Full geometry including unit pull direction at the attachment.
    fn geometry(&self, a: &Vec3, b: &Vec3, ctx: &CableContext) -> CableResult<CableGeometry> {
        let len = self.length(a, b, ctx)?;
        let mut g = CableGeometry::ideal(a, b)?;
        g.geometric = len.geometric;
        g.unstrained = len.unstrained;
        Ok(g)
    }
}
