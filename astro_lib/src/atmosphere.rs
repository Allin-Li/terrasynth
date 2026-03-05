// ── Physical constants ──────────────────────────────────────────────────────
const K_BOLTZ: f64 = 1.380_649e-23; // Boltzmann constant, J/K
const AMU: f64 = 1.660_539e-27; // atomic mass unit, kg
const EARTH_ESCAPE_V: f64 = 11_186.0; // Earth escape velocity, m/s

/// Common gas molecular masses (AMU).
pub struct GasMasses;
impl GasMasses {
    pub const HYDROGEN: f64 = 2.0; // H₂
    pub const HELIUM: f64 = 4.0; // He
    pub const METHANE: f64 = 16.0; // CH₄
    pub const AMMONIA: f64 = 17.0; // NH₃
    pub const WATER: f64 = 18.0; // H₂O
    pub const NITROGEN: f64 = 28.0; // N₂
    pub const OXYGEN: f64 = 32.0; // O₂
    pub const CO2: f64 = 44.0; // CO₂
}

/// Which common gases a planet can retain over geological time.
pub struct AtmosphereRetention {
    pub hydrogen: bool,
    pub helium: bool,
    pub methane: bool,
    pub ammonia: bool,
    pub water_vapor: bool,
    pub nitrogen: bool,
    pub oxygen: bool,
    pub co2: bool,
}

// ── Existing functions ──────────────────────────────────────────────────────

/// Atmospheric scale height in meters: H ≈ 8500 / g_rel
///
/// Approximation for Earth-like atmospheres.
/// g_rel is surface gravity relative to Earth (Earth = 1).
pub fn scale_height(gravity_rel: f64) -> f64 {
    8500.0 / gravity_rel
}

/// Partial pressure of a gas component in atmospheres: P = fraction * P_total
pub fn partial_pressure(fraction: f64, total_pressure_atm: f64) -> f64 {
    fraction * total_pressure_atm
}

// ── Thermal / escape ────────────────────────────────────────────────────────

/// Thermal (RMS) velocity of a gas molecule in m/s: v_th = √(3 k_B T / m).
pub fn thermal_velocity(temperature_k: f64, molecular_mass_amu: f64) -> f64 {
    (3.0 * K_BOLTZ * temperature_k / (molecular_mass_amu * AMU)).sqrt()
}

/// Escape velocity in m/s from relative value (Earth = 1).
pub fn escape_velocity_ms(v_esc_rel: f64) -> f64 {
    v_esc_rel * EARTH_ESCAPE_V
}

/// Can a planet retain a given gas?
///
/// Jeans escape criterion: retained if v_esc > 6 × v_thermal.
pub fn can_retain_gas(
    escape_velocity_rel: f64,
    exosphere_temp_k: f64,
    molecular_mass_amu: f64,
) -> bool {
    let v_esc = escape_velocity_ms(escape_velocity_rel);
    let v_th = thermal_velocity(exosphere_temp_k, molecular_mass_amu);
    v_esc > 6.0 * v_th
}

/// Jeans escape parameter λ = v_esc / v_thermal.
///
/// Values > 6 indicate long-term retention.
pub fn jeans_parameter(
    escape_velocity_rel: f64,
    exosphere_temp_k: f64,
    molecular_mass_amu: f64,
) -> f64 {
    let v_esc = escape_velocity_ms(escape_velocity_rel);
    let v_th = thermal_velocity(exosphere_temp_k, molecular_mass_amu);
    v_esc / v_th
}

/// Retention summary for all common gases at a given escape velocity and exosphere temperature.
pub fn atmosphere_retention(
    escape_velocity_rel: f64,
    exosphere_temp_k: f64,
) -> AtmosphereRetention {
    let ok = |m: f64| can_retain_gas(escape_velocity_rel, exosphere_temp_k, m);
    AtmosphereRetention {
        hydrogen: ok(GasMasses::HYDROGEN),
        helium: ok(GasMasses::HELIUM),
        methane: ok(GasMasses::METHANE),
        ammonia: ok(GasMasses::AMMONIA),
        water_vapor: ok(GasMasses::WATER),
        nitrogen: ok(GasMasses::NITROGEN),
        oxygen: ok(GasMasses::OXYGEN),
        co2: ok(GasMasses::CO2),
    }
}

// ── Temperature ─────────────────────────────────────────────────────────────

/// Equilibrium temperature (K) — blackbody temperature with no atmosphere.
///
/// T_eq = T★ × ((1 − A) R★² / (4 a²))^0.25
///
/// - `star_temp_rel`: stellar temperature in solar units (Sun = 1.0 → 5778 K)
/// - `star_radius_rel`: stellar radius in solar units
/// - `semi_major_axis_au`: orbital semi-major axis in AU
/// - `albedo`: Bond albedo (Earth ≈ 0.3)
pub fn equilibrium_temperature(
    star_temp_rel: f64,
    star_radius_rel: f64,
    semi_major_axis_au: f64,
    albedo: f64,
) -> f64 {
    let t_star_k = star_temp_rel * 5778.0;
    let r_star_au = star_radius_rel * 0.004_650_47; // R☉ in AU
    t_star_k
        * ((1.0 - albedo) * r_star_au.powi(2) / (4.0 * semi_major_axis_au.powi(2)))
            .powf(0.25)
}

/// Rough exosphere temperature estimate (K).
///
/// Earth: T_eq ≈ 255 K, T_exo ≈ 1000 K. Linear approximation.
pub fn exosphere_temperature_estimate(equilibrium_temp_k: f64) -> f64 {
    equilibrium_temp_k * 2.5 + 400.0
}

/// Greenhouse temperature increase (K) — simplified model.
///
/// Base = 33 K (Earth at 1 atm, 0.04 % CO₂).
/// Scales with pressure (broadening) and CO₂ fraction (logarithmic).
pub fn greenhouse_effect(surface_pressure_atm: f64, co2_fraction: f64) -> f64 {
    let base = 33.0;
    let pressure_factor = surface_pressure_atm.powf(0.25);
    let co2_factor = (1.0 + (co2_fraction / 0.0004).ln().max(0.0) * 0.12).max(1.0);
    base * pressure_factor * co2_factor
}

/// Effective surface temperature (K) = T_eq + greenhouse delta.
pub fn surface_temperature(equilibrium_temp_k: f64, greenhouse_delta_k: f64) -> f64 {
    equilibrium_temp_k + greenhouse_delta_k
}

/// Surface pressure estimate (atm) — simple scaling.
///
/// P ∝ g × atmosphere_mass_factor. Factor 1.0 = Earth-like column density.
pub fn surface_pressure_estimate(gravity_rel: f64, atmosphere_mass_factor: f64) -> f64 {
    gravity_rel * atmosphere_mass_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earth_scale_height() {
        assert!((scale_height(1.0) - 8500.0).abs() < 1e-10);
    }

    #[test]
    fn earth_o2_partial_pressure() {
        assert!((partial_pressure(0.21, 1.0) - 0.21).abs() < 1e-10);
    }

    #[test]
    fn earth_equilibrium_temp() {
        // Sun: T_rel=1, R_rel=1, a=1 AU, albedo=0.3 → T_eq ≈ 255 K
        let t = equilibrium_temperature(1.0, 1.0, 1.0, 0.3);
        assert!((t - 255.0).abs() < 5.0);
    }

    #[test]
    fn earth_retains_nitrogen() {
        // Earth v_esc=1.0, exosphere ~1000K, N2=28
        assert!(can_retain_gas(1.0, 1000.0, GasMasses::NITROGEN));
    }

    #[test]
    fn earth_loses_hydrogen() {
        // Earth cannot retain H2 long-term
        assert!(!can_retain_gas(1.0, 1000.0, GasMasses::HYDROGEN));
    }

    #[test]
    fn greenhouse_earth() {
        let g = greenhouse_effect(1.0, 0.0004);
        assert!((g - 33.0).abs() < 1.0);
    }
}
