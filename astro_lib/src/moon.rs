use std::f64::consts::PI;

/// M_earth / M_sun
const EARTH_TO_SOLAR: f64 = 3.003e-6;

/// Constant K for moon orbital period formula, derived from the Moon:
/// T_moon = 27.322 days, a = 60.27 R_earth, M = 1 M_earth
/// K = 27.322 / sqrt(60.27^3) ≈ 0.0584 days
const MOON_PERIOD_K: f64 = 0.0584;

/// Moon mass relative to Earth: M = density_rel * R_rel³
pub fn moon_mass(radius_earth: f64, density_rel: f64) -> f64 {
    density_rel * radius_earth.powi(3)
}

/// Moon surface gravity relative to Earth: g = M / R²
pub fn moon_gravity(mass_earth: f64, radius_earth: f64) -> f64 {
    mass_earth / radius_earth.powi(2)
}

/// Angular size of a moon as seen from the planet surface, in arcminutes
///
/// θ' = (R_moon_km / d_km) * (21600 / π)
pub fn angular_size_arcmin(moon_radius_km: f64, distance_km: f64) -> f64 {
    (moon_radius_km / distance_km) * (21600.0 / PI)
}

/// Hill sphere radius in AU
///
/// r_H = a_AU * (M_planet_solar / (3 * M_star_solar))^(1/3)
pub fn hill_sphere_au(orbital_a_au: f64, planet_mass_earth: f64, star_mass_solar: f64) -> f64 {
    let mass_ratio = (planet_mass_earth * EARTH_TO_SOLAR) / (3.0 * star_mass_solar);
    orbital_a_au * mass_ratio.powf(1.0 / 3.0)
}

/// Hill sphere in planet radii
///
/// Converts the AU result to planet radii using:
/// r_H_planet_radii = r_H_AU * (AU_km / planet_radius_km)
/// where AU_km = 149_600_000 and planet_radius_km = planet_radius_earth * 6_371
pub fn hill_sphere_planet_radii(
    orbital_a_au: f64,
    planet_mass_earth: f64,
    star_mass_solar: f64,
    planet_radius_earth: f64,
) -> f64 {
    let r_h_au = hill_sphere_au(orbital_a_au, planet_mass_earth, star_mass_solar);
    r_h_au * 149_600_000.0 / (planet_radius_earth * 6_371.0)
}

/// Outer boundary of stable moon orbits ≈ 0.5 * Hill sphere radius
pub fn stable_orbit_limit(hill_sphere: f64) -> f64 {
    0.5 * hill_sphere
}

/// Roche limit in planet radii: d_R = 2.44 * (ρ_planet / ρ_moon)^(1/3)
pub fn roche_limit_planet_radii(planet_density_rel: f64, moon_density_rel: f64) -> f64 {
    2.44 * (planet_density_rel / moon_density_rel).powf(1.0 / 3.0)
}

/// Moon orbital period in days
///
/// T_days = K * √(a_earth_radii³ / M_planet_earth)
/// where a_earth_radii = distance in Earth radii (1 R_earth = 6 371 km)
pub fn moon_orbital_period_days(
    semi_major_axis_earth_radii: f64,
    planet_mass_earth: f64,
) -> f64 {
    MOON_PERIOD_K * (semi_major_axis_earth_radii.powi(3) / planet_mass_earth).sqrt()
}

/// Check if two moons at given distances (planet radii) are in a stable configuration.
///
/// Orbits must be separated by a factor of at least 1.3 (empirical criterion for small moons).
pub fn are_moons_stable(inner_distance: f64, outer_distance: f64) -> bool {
    if inner_distance <= 0.0 || outer_distance <= 0.0 {
        return false;
    }
    let ratio = outer_distance / inner_distance;
    ratio >= 1.3
}

/// Check if a moon's orbit is in a valid zone (between Roche limit and stable orbit limit).
pub fn is_moon_orbit_valid(
    distance_planet_radii: f64,
    roche_limit: f64,
    stable_orbit_limit: f64,
) -> bool {
    distance_planet_radii > roche_limit && distance_planet_radii < stable_orbit_limit
}

/// Check if two moons are near a mean-motion resonance (potentially destabilizing).
///
/// Returns true if the period ratio is within 5% of 2:1, 3:2, 3:1, 4:3, or 5:3.
pub fn near_resonance(period_inner_days: f64, period_outer_days: f64) -> bool {
    if period_inner_days <= 0.0 || period_outer_days <= 0.0 {
        return false;
    }
    let ratio = period_outer_days / period_inner_days;
    let resonances = [2.0, 1.5, 3.0, 4.0 / 3.0, 5.0 / 3.0];
    resonances.iter().any(|&r| (ratio - r).abs() < 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moon_angular_size_approx_31_arcmin() {
        // R_moon = 1736 km, d = 384 400 km → ~31 arcmin
        let size = angular_size_arcmin(1736.0, 384_400.0);
        assert!((size - 31.05).abs() < 0.5);
    }

    #[test]
    fn moon_orbital_period_27_days() {
        // Moon: a = 60.27 R_earth, M_planet = 1 M_earth → 27.32 days
        let t = moon_orbital_period_days(60.27, 1.0);
        assert!((t - 27.32).abs() < 0.1);
    }

    /// Onsu (New Terra): a=53 R_earth, M_planet=1.18 → ~20.7 days (table)
    #[test]
    fn onsu_orbital_period() {
        let t = moon_orbital_period_days(53.0, 1.18);
        assert!((t - 20.7).abs() < 0.2);
    }
}
