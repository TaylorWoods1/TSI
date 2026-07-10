//! Error types for spyder-core.

use thiserror::Error;

/// Result alias for spyder-core operations.
pub type Result<T> = std::result::Result<T, SpyderError>;

/// Typed errors surfaced by the kinematics / config stack.
#[derive(Debug, Error)]
pub enum SpyderError {
    /// Invalid robot or preset configuration.
    #[error("invalid configuration: {0}")]
    Config(String),
    /// Degenerate or impossible geometry.
    #[error("geometry error: {0}")]
    Geometry(String),
    /// Forward kinematics failed to converge.
    #[error("FK did not converge (residual={residual}, iterations={iterations})")]
    FkNonConvergence {
        /// Final residual norm.
        residual: f64,
        /// Iterations attempted.
        iterations: usize,
    },
    /// No feasible positive tension solution within bounds.
    #[error("wrench infeasible at pose")]
    InfeasibleWrench,
    /// Structure matrix rank-deficient for the requested solve.
    #[error("singular structure matrix")]
    SingularStructure,
    /// Cable model failure (ideal/pulley/sag).
    #[error("cable model error: {0}")]
    Model(String),
}
