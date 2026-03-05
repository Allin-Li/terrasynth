use crate::error::StarErr;
use crate::star::{habitable_zone, luminosity, HabitableZone};

/// Binary star orbit type for planets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOrbitType {
    /// S-type: planet orbits one star, the other is distant.
    SType,
    /// P-type (circumbinary): planet orbits both stars.
    PType,
}

impl std::fmt::Display for BinaryOrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOrbitType::SType => write!(f, "S-type (circumstellar)"),
            BinaryOrbitType::PType => write!(f, "P-type (circumbinary)"),
        }
    }
}

/// Binary orbital period in years (Kepler's 3rd law):
/// T = sqrt(a^3 / (M_A + M_B))
pub fn binary_orbital_period(
    separation_au: f64,
    mass_a_solar: f64,
    mass_b_solar: f64,
) -> f64 {
    (separation_au.powi(3) / (mass_a_solar + mass_b_solar)).sqrt()
}

/// Combined luminosity of a binary system (solar units).
pub fn combined_luminosity(
    mass_a_solar: f64,
    mass_b_solar: f64,
) -> Result<f64, StarErr> {
    let la = luminosity(mass_a_solar)?;
    let lb = luminosity(mass_b_solar)?;
    Ok(la + lb)
}

/// Habitable zone for the combined luminosity of a binary system.
pub fn binary_habitable_zone(
    mass_a_solar: f64,
    mass_b_solar: f64,
) -> Result<HabitableZone, StarErr> {
    let l = combined_luminosity(mass_a_solar, mass_b_solar)?;
    Ok(habitable_zone(l))
}

/// S-type stability: maximum semi-major axis for a planet orbiting one star
/// in a binary system (Holman & Wiegert 1999 fitting formula).
pub fn s_type_critical_radius(
    binary_separation_au: f64,
    binary_eccentricity: f64,
    primary_mass_solar: f64,
    secondary_mass_solar: f64,
) -> f64 {
    let total = primary_mass_solar + secondary_mass_solar;
    let mu = secondary_mass_solar / total;
    let a_crit = binary_separation_au
        * (0.464 - 0.380 * mu)
        * (1.0 - 1.767 * binary_eccentricity);
    a_crit.max(0.0)
}

/// P-type stability: minimum semi-major axis for a circumbinary planet
/// (Holman & Wiegert 1999 fitting formula).
pub fn p_type_critical_radius(
    binary_separation_au: f64,
    binary_eccentricity: f64,
    primary_mass_solar: f64,
    secondary_mass_solar: f64,
) -> f64 {
    let total = primary_mass_solar + secondary_mass_solar;
    let mu = secondary_mass_solar / total;
    let e = binary_eccentricity;
    binary_separation_au * (1.60 + 5.10 * e + 4.12 * mu - 2.22 * mu.powi(2) - 2.95 * e * mu)
}

/// Check if a planet's orbit is stable in the binary system.
pub fn is_orbit_stable_binary(
    orbit_type: BinaryOrbitType,
    planet_semi_major_au: f64,
    binary_separation_au: f64,
    binary_eccentricity: f64,
    primary_mass_solar: f64,
    secondary_mass_solar: f64,
) -> bool {
    match orbit_type {
        BinaryOrbitType::SType => {
            planet_semi_major_au
                < s_type_critical_radius(
                    binary_separation_au,
                    binary_eccentricity,
                    primary_mass_solar,
                    secondary_mass_solar,
                )
        }
        BinaryOrbitType::PType => {
            planet_semi_major_au
                > p_type_critical_radius(
                    binary_separation_au,
                    binary_eccentricity,
                    primary_mass_solar,
                    secondary_mass_solar,
                )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_mass_binary_period() {
        // Two solar-mass stars at 20 AU: T = sqrt(20^3 / 2) ≈ 63.2 yr
        let p = binary_orbital_period(20.0, 1.0, 1.0);
        assert!((p - 63.2).abs() < 0.5);
    }

    #[test]
    fn s_type_limit() {
        // Equal mass, a_bin=20 AU, e=0 → critical ~ 0.464 - 0.190 = 0.274 * 20 ≈ 5.48 AU
        let crit = s_type_critical_radius(20.0, 0.0, 1.0, 1.0);
        assert!(crit > 5.0 && crit < 6.0);
    }

    #[test]
    fn p_type_limit() {
        // Equal mass, a_bin=1 AU, e=0 → critical ~ 1.60 + 2.06 - 0.555 ≈ 3.1 AU
        let crit = p_type_critical_radius(1.0, 0.0, 1.0, 1.0);
        assert!(crit > 2.5 && crit < 4.0);
    }
}
