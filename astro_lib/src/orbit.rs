#[derive(Debug, Clone, Copy)]
pub struct OrbitalPeriod {
    pub years: f64,
    pub days: f64,
}

/// Aphelion (farthest point) in AU: r_a = a(1 + e)
pub fn aphelion(semi_major_axis: f64, eccentricity: f64) -> f64 {
    semi_major_axis * (1.0 + eccentricity)
}

/// Perihelion (closest point) in AU: r_p = a(1 - e)
pub fn perihelion(semi_major_axis: f64, eccentricity: f64) -> f64 {
    semi_major_axis * (1.0 - eccentricity)
}

/// Orbital period from Kepler's third law: T = √(a³ / M_star)
pub fn orbital_period(semi_major_axis_au: f64, star_mass_solar: f64) -> OrbitalPeriod {
    let years = (semi_major_axis_au.powi(3) / star_mass_solar).sqrt();
    OrbitalPeriod {
        years,
        days: years * 365.25,
    }
}

/// Mean orbital velocity relative to Earth: v = √(M / a)
pub fn orbital_velocity(semi_major_axis_au: f64, star_mass_solar: f64) -> f64 {
    (star_mass_solar / semi_major_axis_au).sqrt()
}

/// Estimated orbital eccentricity from number of planets in the system: e = 0.584 * N^-1.2
pub fn eccentricity_from_n_planets(n_planets: u32) -> f64 {
    0.584 * (n_planets as f64).powf(-1.2)
}

/// Tropic latitude equals axial tilt (degrees)
pub fn tropic_latitude(axial_tilt_deg: f64) -> f64 {
    axial_tilt_deg
}

/// Polar circle latitude: φ = 90° − axial_tilt
pub fn polar_circle(axial_tilt_deg: f64) -> f64 {
    90.0 - axial_tilt_deg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_orbital_period_is_one_year() {
        let p = orbital_period(1.0, 1.0);
        assert!((p.years - 1.0).abs() < 1e-10);
        assert!((p.days - 365.25).abs() < 1e-10);
    }

    /// New Terra: a=1.08, e=0.015 → aphelion≈1.096, perihelion≈1.064
    #[test]
    fn new_terra_orbit() {
        assert!((aphelion(1.08, 0.015) - 1.096).abs() < 0.001);
        assert!((perihelion(1.08, 0.015) - 1.064).abs() < 0.001);
    }

    /// New Terra: a=1.08, M_star=1.03 → T≈1.106 years (table: 1.106)
    #[test]
    fn new_terra_period() {
        let p = orbital_period(1.08, 1.03);
        assert!((p.years - 1.106).abs() < 0.002);
    }

    #[test]
    fn earth_tropic_and_polar() {
        assert!((tropic_latitude(23.4) - 23.4).abs() < 1e-10);
        assert!((polar_circle(23.4) - 66.6).abs() < 1e-10);
    }
}
