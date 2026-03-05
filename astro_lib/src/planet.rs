/// Planet classification based on mass (Earth masses).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetType {
    Rocky,       // < 2 M⊕
    SubNeptune,  // 2–10 M⊕
    GasGiant,    // 10–300 M⊕
    SuperJovian, // > 300 M⊕
}

impl std::fmt::Display for PlanetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanetType::Rocky => write!(f, "Rocky"),
            PlanetType::SubNeptune => write!(f, "Sub-Neptune"),
            PlanetType::GasGiant => write!(f, "Gas Giant"),
            PlanetType::SuperJovian => write!(f, "Super-Jovian"),
        }
    }
}

/// Classify a planet by mass (Earth masses).
pub fn planet_type(mass_earth: f64) -> PlanetType {
    if mass_earth < 2.0 {
        PlanetType::Rocky
    } else if mass_earth < 10.0 {
        PlanetType::SubNeptune
    } else if mass_earth < 300.0 {
        PlanetType::GasGiant
    } else {
        PlanetType::SuperJovian
    }
}

/// Whether this planet type has a solid surface.
pub fn has_solid_surface(ptype: PlanetType) -> bool {
    matches!(ptype, PlanetType::Rocky)
}

/// Approximate radius from mass for rocky/terrestrial planets (relative to Earth): R ≈ M^0.3
///
/// Empirical formula valid for 0.1–10 Earth masses.
pub fn planet_radius_from_mass(mass: f64) -> f64 {
    mass.powf(0.3)
}

/// Radius estimate that accounts for planet type (Chen & Kipping 2017 inspired).
///
/// - Rocky (< 2 M⊕): R ~ M^0.3
/// - Sub-Neptune (2–10 M⊕): R ~ M^0.55
/// - Gas Giant (10–300 M⊕): R ~ 3.9 * M^0.06 (roughly Jupiter-sized)
/// - Super-Jovian (> 300 M⊕): R ~ 11.2 * (M/318)^−0.04 (degeneracy shrinkage)
pub fn planet_radius_auto(mass_earth: f64) -> f64 {
    match planet_type(mass_earth) {
        PlanetType::Rocky => mass_earth.powf(0.3),
        PlanetType::SubNeptune => mass_earth.powf(0.55),
        PlanetType::GasGiant => 3.9 * mass_earth.powf(0.06),
        PlanetType::SuperJovian => {
            let m_jup = mass_earth / 318.0;
            11.2 * m_jup.powf(-0.04)
        }
    }
}

/// Surface gravity relative to Earth: g = M / R²
pub fn gravity(mass: f64, radius: f64) -> f64 {
    mass / radius.powi(2)
}

/// Density relative to Earth: ρ = M / R³
pub fn density(mass: f64, radius: f64) -> f64 {
    mass / radius.powi(3)
}

/// Escape velocity relative to Earth: v_esc = √(M / R)
pub fn escape_velocity(mass: f64, radius: f64) -> f64 {
    (mass / radius).sqrt()
}

/// Surface area relative to Earth: S = R²
pub fn surface_area(radius: f64) -> f64 {
    radius.powi(2)
}

/// Volume relative to Earth: V = R³
pub fn volume(radius: f64) -> f64 {
    radius.powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_unit_values() {
        assert!((gravity(1.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((density(1.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((escape_velocity(1.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((surface_area(1.0) - 1.0).abs() < 1e-10);
        assert!((volume(1.0) - 1.0).abs() < 1e-10);
    }

    /// New Terra: M=1.18, R=1.05 → g≈1.07, ρ≈1.02 (table: g=1.1, ρ=1.05)
    #[test]
    fn new_terra_gravity() {
        let g = gravity(1.18, 1.05);
        assert!((g - 1.07).abs() < 0.02);
    }
}
