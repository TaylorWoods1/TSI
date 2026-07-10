//! CDPR restraint classification (Ming–Higuchi / Verhoeven).

/// Restraint class from cable count `m` vs platform DOF `n`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestraintClass {
    /// Incompletely restrained: `m < n` (cannot resist arbitrary wrenches).
    Irpm,
    /// Completely restrained: `m == n`.
    Crpm,
    /// Redundantly restrained: `m > n`.
    Rrpm,
}

impl RestraintClass {
    /// Stable short name (`IRPM` / `CRPM` / `RRPM`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Irpm => "IRPM",
            Self::Crpm => "CRPM",
            Self::Rrpm => "RRPM",
        }
    }
}

impl std::fmt::Display for RestraintClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classify a CDPR by cable count and DOF.
pub fn classify_restraint(m: usize, n: usize) -> Result<RestraintClass, String> {
    if m == 0 {
        return Err("need at least one cable".into());
    }
    if n == 0 {
        return Err("DOF n must be > 0".into());
    }
    Ok(if m < n {
        RestraintClass::Irpm
    } else if m == n {
        RestraintClass::Crpm
    } else {
        RestraintClass::Rrpm
    })
}

/// Classify using cable count and measured structure-matrix rank.
pub fn classify_restraint_ranked(m: usize, n: usize, rank: usize) -> Result<RestraintClass, String> {
    if rank < n {
        Ok(RestraintClass::Irpm)
    } else if m == n && rank == n {
        Ok(RestraintClass::Crpm)
    } else if m > n && rank >= n {
        Ok(RestraintClass::Rrpm)
    } else {
        classify_restraint(m, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect4_point_mass_is_rrpm() {
        assert_eq!(classify_restraint(4, 3).unwrap(), RestraintClass::Rrpm);
    }

    #[test]
    fn triangle_point_mass_is_crpm() {
        assert_eq!(classify_restraint(3, 3).unwrap(), RestraintClass::Crpm);
    }

    #[test]
    fn two_cables_point_mass_is_irpm() {
        assert_eq!(classify_restraint(2, 3).unwrap(), RestraintClass::Irpm);
    }

    #[test]
    fn eight_cable_platform_is_rrpm() {
        assert_eq!(classify_restraint(8, 6).unwrap(), RestraintClass::Rrpm);
    }
}
