//! Statics: structure matrix, tension distribution, feasibility.

#![deny(missing_docs)]

pub mod classify;
pub mod structure;
pub mod tension;

pub use classify::{classify_restraint, RestraintClass};
pub use structure::{structure_matrix_3, structure_matrix_6, StructureError, Vec3};
pub use tension::{
    closed_form_tensions, is_wrench_feasible, qp_tensions, solve_tensions, TensionError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use nalgebra::DVector;

    #[test]
    fn four_cable_point_mass_center_gravity_positive_tensions() {
        let exits = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
        ];
        let p = Vec3::new(0.0, 0.0, 0.0);
        let mut units = Vec::new();
        for a in &exits {
            let d = a - p;
            units.push(d / d.norm());
        }
        let a = structure_matrix_3(&units).unwrap();
        let mg = 10.0;
        let w = DVector::from_vec(vec![0.0, 0.0, -mg]);
        let f = closed_form_tensions(&a, &w, 0.5, 100.0).expect("feasible");
        assert_eq!(f.len(), 4);
        for fi in f.iter() {
            assert!(*fi > 0.0, "tension {fi} should be positive");
        }
        let mut fz = 0.0;
        for (fi, u) in f.iter().zip(units.iter()) {
            fz += fi * u.z;
        }
        assert_relative_eq!(fz, mg, epsilon = 1e-6);
        assert!(is_wrench_feasible(&a, &w, 0.5, 100.0));
    }
}
