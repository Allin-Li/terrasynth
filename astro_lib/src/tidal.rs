// ── Physical constants (SI) ─────────────────────────────────────────────────
const AU_M: f64 = 1.496e11; // astronomical unit, m
const EARTH_RADIUS_M: f64 = 6.371e6; // Earth radius, m
const EARTH_MASS_KG: f64 = 5.972e24; // Earth mass, kg
const SOLAR_MASS_KG: f64 = 1.989e30; // Solar mass, kg

/// Rigidity of rocky bodies, N/m²
pub const RIGIDITY_ROCKY: f64 = 3.0e10;
/// Rigidity of icy bodies, N/m²
pub const RIGIDITY_ICY: f64 = 4.0e9;

/// Tidal dissipation quality factor, typical for terrestrial bodies.
const DISSIPATION_Q: f64 = 100.0;

/// Tidal locking timescale in years — order-of-magnitude estimate
/// (Gladman et al. 1996, as popularized on the "Tidal locking" survey formula):
///
/// t ≈ 6 · a⁶ · R · μ · Q / (m_sat · m_host²) · 10¹⁰ yr   (SI inputs)
///
/// Assumes an initial rotation period of ~12 h; real values scatter by
/// a couple of orders of magnitude, so treat the result as indicative.
pub fn lock_time_years_si(
    semi_major_m: f64,
    satellite_radius_m: f64,
    satellite_mass_kg: f64,
    host_mass_kg: f64,
    rigidity: f64,
) -> f64 {
    6.0 * semi_major_m.powi(6) * satellite_radius_m * rigidity * DISSIPATION_Q
        / (satellite_mass_kg * host_mass_kg.powi(2))
        * 1e10
}

/// Tidal locking timescale for a planet around its star, in years.
///
/// Inputs in the calculator's relative units:
/// planet mass/radius in Earth units, star mass in solar units, orbit in AU.
pub fn planet_lock_time_years(
    planet_mass_earth: f64,
    planet_radius_earth: f64,
    star_mass_solar: f64,
    semi_major_au: f64,
    rigidity: f64,
) -> f64 {
    lock_time_years_si(
        semi_major_au * AU_M,
        planet_radius_earth * EARTH_RADIUS_M,
        planet_mass_earth * EARTH_MASS_KG,
        star_mass_solar * SOLAR_MASS_KG,
        rigidity,
    )
}

/// Tidal locking timescale for a moon around its planet, in years.
///
/// Moon mass/radius in Earth units, planet mass in Earth units,
/// orbital distance in planet radii (planet radius in Earth units).
pub fn moon_lock_time_years(
    moon_mass_earth: f64,
    moon_radius_earth: f64,
    planet_mass_earth: f64,
    distance_planet_radii: f64,
    planet_radius_earth: f64,
    rigidity: f64,
) -> f64 {
    lock_time_years_si(
        distance_planet_radii * planet_radius_earth * EARTH_RADIUS_M,
        moon_radius_earth * EARTH_RADIUS_M,
        moon_mass_earth * EARTH_MASS_KG,
        planet_mass_earth * EARTH_MASS_KG,
        rigidity,
    )
}

/// Whether the body has had enough time to become tidally locked.
pub fn is_tidally_locked(lock_time_years: f64, system_age_years: f64) -> bool {
    lock_time_years < system_age_years
}

#[cfg(test)]
mod tests {
    use super::*;

    const EARTH_AGE_YEARS: f64 = 4.6e9;

    #[test]
    fn earth_is_not_locked() {
        let t = planet_lock_time_years(1.0, 1.0, 1.0, 1.0, RIGIDITY_ROCKY);
        // Known estimate ~10¹¹–10¹² years
        assert!(t > 1e11 && t < 1e13);
        assert!(!is_tidally_locked(t, EARTH_AGE_YEARS));
    }

    #[test]
    fn proxima_b_is_locked() {
        // Proxima Centauri: M=0.122 M☉; planet b: ~1.17 M⊕, a=0.0485 AU
        let t = planet_lock_time_years(1.17, 1.05, 0.122, 0.0485, RIGIDITY_ROCKY);
        assert!(t < 1e7);
        assert!(is_tidally_locked(t, EARTH_AGE_YEARS));
    }

    #[test]
    fn mercury_is_not_locked() {
        // Mercury: 0.055 M⊕, 0.383 R⊕, a=0.387 AU (real Mercury is in a 3:2
        // resonance, not locked — eccentricity keeps it spinning)
        let t = planet_lock_time_years(0.055, 0.383, 1.0, 0.387, RIGIDITY_ROCKY);
        assert!(!is_tidally_locked(t, EARTH_AGE_YEARS));
    }

    #[test]
    fn moon_is_locked() {
        // Moon: 0.0123 M⊕, 0.273 R⊕, at 60.3 planet radii around Earth
        let t = moon_lock_time_years(0.0123, 0.273, 1.0, 60.3, 1.0, RIGIDITY_ROCKY);
        assert!(is_tidally_locked(t, EARTH_AGE_YEARS));
    }
}
