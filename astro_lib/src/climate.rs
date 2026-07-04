use std::f64::consts::PI;

/// Fraction of the equator-pole radiative imbalance smoothed out by
/// atmospheric/oceanic heat transport. 0 = pure radiative balance,
/// 1 = perfectly mixed. Calibrated so Earth gives ≈ +25 °C at the
/// equator and ≈ −15 °C at the poles.
pub const HEAT_TRANSPORT_EARTH: f64 = 0.3;

/// How much of the instantaneous radiative swing actually shows up as
/// seasonal temperature change (thermal inertia damping).
/// Calibrated so Earth gets a boreal belt at 60° and ≈ −30 °C Arctic winters.
pub const SEASONAL_DAMPING_EARTH: f64 = 0.35;

/// Orbit samples per year for annual means and seasonal extremes.
const ORBIT_STEPS: usize = 144;

/// Day-averaged insolation at a latitude for a given solar declination,
/// as a fraction of the solar constant.
///
/// q = (1/π)(h₀ sin φ sin δ + cos φ cos δ sin h₀), where h₀ is the sunset
/// hour angle: cos h₀ = −tan φ · tan δ (clamped for polar day/night).
pub fn daily_insolation_rel(latitude_deg: f64, declination_deg: f64) -> f64 {
    let phi = latitude_deg.to_radians();
    let delta = declination_deg.to_radians();
    let cos_h0 = (-phi.tan() * delta.tan()).clamp(-1.0, 1.0);
    let h0 = cos_h0.acos();
    let q = (h0 * phi.sin() * delta.sin() + phi.cos() * delta.cos() * h0.sin()) / PI;
    q.max(0.0)
}

/// Solar declination (deg) at orbital longitude λ (rad, 0 = spring equinox):
/// sin δ = sin ε · sin λ. Retrograde tilts (> 90°) fold automatically.
pub fn solar_declination_deg(axial_tilt_deg: f64, orbital_longitude_rad: f64) -> f64 {
    (axial_tilt_deg.to_radians().sin() * orbital_longitude_rad.sin()).asin().to_degrees()
}

/// Solve Kepler's equation M = E − e·sin E for the eccentric anomaly (rad)
/// by Newton iteration.
fn eccentric_anomaly(mean_anomaly: f64, eccentricity: f64) -> f64 {
    let mut ea = if eccentricity < 0.8 { mean_anomaly } else { PI };
    for _ in 0..10 {
        ea -= (ea - eccentricity * ea.sin() - mean_anomaly) / (1.0 - eccentricity * ea.cos());
    }
    ea
}

/// Point on an eccentric orbit at mean anomaly M (i.e. uniform in time):
/// returns (orbital longitude λ from spring equinox, flux factor (a/r)²).
/// `perihelion_deg` is the orbital longitude of perihelion (0 = spring
/// equinox, 90 = northern summer solstice).
fn orbit_sample(mean_anomaly: f64, eccentricity: f64, perihelion_deg: f64) -> (f64, f64) {
    let e = eccentricity.clamp(0.0, 0.95);
    let ea = eccentric_anomaly(mean_anomaly, e);
    let true_anomaly =
        2.0 * ((1.0 + e).sqrt() * (ea / 2.0).sin()).atan2((1.0 - e).sqrt() * (ea / 2.0).cos());
    let r_rel = 1.0 - e * ea.cos(); // r/a
    (true_anomaly + perihelion_deg.to_radians(), 1.0 / (r_rel * r_rel))
}

/// Annual (time-averaged) mean insolation at a latitude, as a fraction of
/// the solar constant at the semi-major axis. For a circular orbit the
/// global mean is exactly 1/4; eccentricity boosts it by 1/√(1−e²).
pub fn annual_mean_insolation_rel(
    latitude_deg: f64,
    axial_tilt_deg: f64,
    eccentricity: f64,
    perihelion_deg: f64,
) -> f64 {
    let mut sum = 0.0;
    for i in 0..ORBIT_STEPS {
        let m = 2.0 * PI * (i as f64 + 0.5) / ORBIT_STEPS as f64;
        let (lambda, flux) = orbit_sample(m, eccentricity, perihelion_deg);
        let decl = solar_declination_deg(axial_tilt_deg, lambda);
        sum += daily_insolation_rel(latitude_deg, decl) * flux;
    }
    sum / ORBIT_STEPS as f64
}

/// Köppen-style climate zone from warmest/coldest season temperatures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClimateZone {
    /// Warmest season below 0 °C — permanent ice.
    IceCap,
    /// Warmest season 0–10 °C.
    Tundra,
    /// Cold winters (< −3 °C) but real summers (> 10 °C).
    Boreal,
    /// Mild winters, warm summers.
    Temperate,
    /// Coldest season ≥ 18 °C.
    Tropical,
    /// Coldest season ≥ 35 °C — hostile heat year-round.
    Scorched,
}

impl std::fmt::Display for ClimateZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ClimateZone::IceCap => "Ice cap",
            ClimateZone::Tundra => "Tundra",
            ClimateZone::Boreal => "Boreal",
            ClimateZone::Temperate => "Temperate",
            ClimateZone::Tropical => "Tropical",
            ClimateZone::Scorched => "Scorched",
        };
        write!(f, "{s}")
    }
}

/// Classify a latitude band by its seasonal extremes (°C).
pub fn climate_zone(summer_c: f64, winter_c: f64) -> ClimateZone {
    let warm = summer_c.max(winter_c);
    let cold = summer_c.min(winter_c);
    if warm < 0.0 {
        ClimateZone::IceCap
    } else if warm < 10.0 {
        ClimateZone::Tundra
    } else if cold >= 35.0 {
        ClimateZone::Scorched
    } else if cold >= 18.0 {
        ClimateZone::Tropical
    } else if cold >= -3.0 {
        ClimateZone::Temperate
    } else {
        ClimateZone::Boreal
    }
}

/// Display hex color for a climate zone (for UI bands/badges).
pub fn zone_display_color(zone: ClimateZone) -> &'static str {
    match zone {
        ClimateZone::IceCap => "#dfeefa",
        ClimateZone::Tundra => "#7fb3d5",
        ClimateZone::Boreal => "#16a085",
        ClimateZone::Temperate => "#2ecc71",
        ClimateZone::Tropical => "#e67e22",
        ClimateZone::Scorched => "#e74c3c",
    }
}

/// Climate summary for one latitude band.
#[derive(Debug, Clone, Copy)]
pub struct LatitudeClimate {
    /// Signed latitude: positive = north, negative = south.
    pub latitude_deg: f64,
    /// Annual mean surface temperature, K
    pub annual_k: f64,
    /// Warm-season temperature, K
    pub summer_k: f64,
    /// Cold-season temperature, K
    pub winter_k: f64,
    pub zone: ClimateZone,
}

/// Radiative-equilibrium temperature for a given insolation ratio,
/// smoothed by heat transport: T = T_mean · ((1−f)·I/Ī + f)^¼.
fn balance_temp_k(mean_temp_k: f64, insolation_ratio: f64, heat_transport: f64) -> f64 {
    mean_temp_k * ((1.0 - heat_transport) * insolation_ratio + heat_transport).powf(0.25)
}

/// Energy-balance climate for one latitude (signed: negative = south).
///
/// - `mean_surface_temp_k`: the planet's global mean surface temperature
///   (equilibrium + greenhouse), which this model redistributes.
/// - `eccentricity`, `perihelion_deg`: orbit shape and orientation; the
///   perihelion longitude is measured from the northern spring equinox,
///   so 90° puts perihelion at northern summer solstice.
/// - `heat_transport`: 0–1, see [`HEAT_TRANSPORT_EARTH`].
/// - `seasonal_damping`: 0–1, see [`SEASONAL_DAMPING_EARTH`].
///
/// Seasonal extremes come from scanning the whole year (uniform in time
/// via Kepler's equation), so perihelion timing shifts and skews seasons
/// between hemispheres. Assumes a rotating planet (not tidally locked).
pub fn latitude_climate(
    latitude_deg: f64,
    axial_tilt_deg: f64,
    mean_surface_temp_k: f64,
    eccentricity: f64,
    perihelion_deg: f64,
    heat_transport: f64,
    seasonal_damping: f64,
) -> LatitudeClimate {
    const GLOBAL_MEAN_INSOLATION: f64 = 0.25;
    let mut q = [0.0f64; ORBIT_STEPS];
    for (i, qi) in q.iter_mut().enumerate() {
        let m = 2.0 * PI * (i as f64 + 0.5) / ORBIT_STEPS as f64;
        let (lambda, flux) = orbit_sample(m, eccentricity, perihelion_deg);
        let decl = solar_declination_deg(axial_tilt_deg, lambda);
        *qi = daily_insolation_rel(latitude_deg, decl) * flux;
    }
    let annual_ratio = q.iter().sum::<f64>() / ORBIT_STEPS as f64 / GLOBAL_MEAN_INSOLATION;
    let annual_k = balance_temp_k(mean_surface_temp_k, annual_ratio, heat_transport);

    // Instantaneous radiative response at each point of the year, damped
    // by thermal inertia toward the annual mean.
    let mut summer_k = f64::NEG_INFINITY;
    let mut winter_k = f64::INFINITY;
    for qi in q {
        let instant =
            balance_temp_k(mean_surface_temp_k, qi / GLOBAL_MEAN_INSOLATION, heat_transport);
        let t = annual_k + seasonal_damping * (instant - annual_k);
        summer_k = summer_k.max(t);
        winter_k = winter_k.min(t);
    }

    LatitudeClimate {
        latitude_deg,
        annual_k,
        summer_k,
        winter_k,
        zone: climate_zone(summer_k - 273.15, winter_k - 273.15),
    }
}

/// Climate for both hemispheres, −90°…+90° in 15° steps (south to north),
/// with Earth-calibrated transport and damping.
pub fn climate_bands(
    axial_tilt_deg: f64,
    mean_surface_temp_k: f64,
    eccentricity: f64,
    perihelion_deg: f64,
) -> Vec<LatitudeClimate> {
    (-6..=6)
        .map(|i| {
            latitude_climate(
                (i * 15) as f64,
                axial_tilt_deg,
                mean_surface_temp_k,
                eccentricity,
                perihelion_deg,
                HEAT_TRANSPORT_EARTH,
                SEASONAL_DAMPING_EARTH,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EARTH_TILT: f64 = 23.4;
    const EARTH_MEAN_K: f64 = 288.0;

    fn earth_lat(lat: f64) -> LatitudeClimate {
        latitude_climate(
            lat,
            EARTH_TILT,
            EARTH_MEAN_K,
            0.0,
            0.0,
            HEAT_TRANSPORT_EARTH,
            SEASONAL_DAMPING_EARTH,
        )
    }

    #[test]
    fn global_mean_insolation_is_quarter() {
        // cos-weighted average over latitudes must equal S0/4
        let (mut sum, mut w) = (0.0, 0.0);
        for i in 0..90 {
            let lat = i as f64 + 0.5;
            let weight = lat.to_radians().cos();
            sum += annual_mean_insolation_rel(lat, EARTH_TILT, 0.0, 0.0) * weight;
            w += weight;
        }
        assert!((sum / w - 0.25).abs() < 0.003);
    }

    #[test]
    fn earth_equator_annual_insolation() {
        // Observed TOA annual mean at the equator ≈ 0.30 S0
        let i = annual_mean_insolation_rel(0.0, EARTH_TILT, 0.0, 0.0);
        assert!((i - 0.30).abs() < 0.01);
    }

    #[test]
    fn earth_equator_is_tropical() {
        let c = earth_lat(0.0);
        let annual_c = c.annual_k - 273.15;
        assert!(annual_c > 20.0 && annual_c < 30.0);
        assert_eq!(c.zone, ClimateZone::Tropical);
    }

    #[test]
    fn earth_mid_latitude_is_temperate() {
        let c = earth_lat(45.0);
        let summer_c = c.summer_k - 273.15;
        let winter_c = c.winter_k - 273.15;
        assert!(summer_c > 14.0 && summer_c < 25.0);
        assert!(winter_c > -8.0 && winter_c < 6.0);
        assert_eq!(c.zone, ClimateZone::Temperate);
    }

    #[test]
    fn earth_60_is_boreal() {
        // Real 60°N (taiga belt): short real summers, cold winters
        let c = earth_lat(60.0);
        assert_eq!(c.zone, ClimateZone::Boreal);
    }

    #[test]
    fn earth_pole_is_frozen() {
        let c = earth_lat(90.0);
        let summer_c = c.summer_k - 273.15;
        let winter_c = c.winter_k - 273.15;
        // Arctic: summers near freezing, winters ≈ −30 °C
        assert!(summer_c > -6.0 && summer_c < 6.0);
        assert!(winter_c > -40.0 && winter_c < -20.0);
        assert!(matches!(c.zone, ClimateZone::IceCap | ClimateZone::Tundra));
    }

    #[test]
    fn zero_tilt_kills_seasons() {
        let c = latitude_climate(
            45.0,
            0.0,
            EARTH_MEAN_K,
            0.0,
            0.0,
            HEAT_TRANSPORT_EARTH,
            SEASONAL_DAMPING_EARTH,
        );
        assert!((c.summer_k - c.winter_k).abs() < 0.1);
    }

    #[test]
    fn extreme_tilt_pole_beats_equator() {
        // Above ~54° tilt the poles receive more annual energy than the equator
        let pole = annual_mean_insolation_rel(90.0, 90.0, 0.0, 0.0);
        let equator = annual_mean_insolation_rel(0.0, 90.0, 0.0, 0.0);
        assert!(pole > equator);
    }

    #[test]
    fn retrograde_tilt_folds() {
        // 177° retrograde tilt behaves like 3°
        let a = annual_mean_insolation_rel(45.0, 177.0, 0.0, 0.0);
        let b = annual_mean_insolation_rel(45.0, 3.0, 0.0, 0.0);
        assert!((a - b).abs() < 1e-9);
    }

    #[test]
    fn circular_orbit_is_hemisphere_symmetric() {
        for lat in [15.0, 45.0, 75.0] {
            let n = earth_lat(lat);
            let s = earth_lat(-lat);
            assert!((n.summer_k - s.summer_k).abs() < 1e-6);
            assert!((n.winter_k - s.winter_k).abs() < 1e-6);
        }
    }

    #[test]
    fn eccentricity_boosts_mean_flux() {
        // Time-averaged (a/r)² = 1/√(1−e²): the cos-weighted global mean
        // insolation must rise from 0.25 to 0.25/√(1−e²).
        let e: f64 = 0.5;
        let (mut sum, mut w) = (0.0, 0.0);
        for i in 0..90 {
            let lat = i as f64 + 0.5;
            let weight = lat.to_radians().cos();
            sum += annual_mean_insolation_rel(lat, EARTH_TILT, e, 40.0) * weight;
            w += weight;
        }
        let expected = 0.25 / (1.0 - e * e).sqrt();
        assert!((sum / w - expected).abs() < 0.005);
    }

    #[test]
    fn mars_south_gets_harsher_seasons() {
        // Mars: e = 0.0934, perihelion just before southern summer
        // solstice (λ_p ≈ 251°) → southern seasons are more extreme.
        let mars = |lat: f64| {
            latitude_climate(
                lat,
                25.2,
                210.0,
                0.0934,
                251.0,
                HEAT_TRANSPORT_EARTH,
                SEASONAL_DAMPING_EARTH,
            )
        };
        let north = mars(45.0);
        let south = mars(-45.0);
        assert!(south.summer_k > north.summer_k);
        assert!(south.winter_k < north.winter_k);
    }

    #[test]
    fn perihelion_at_north_summer_warms_north() {
        // Perihelion at λ = 90° (northern summer solstice) on a fat
        // orbit: the north gets the hotter summer.
        let c = |lat: f64| {
            latitude_climate(
                lat,
                EARTH_TILT,
                EARTH_MEAN_K,
                0.3,
                90.0,
                HEAT_TRANSPORT_EARTH,
                SEASONAL_DAMPING_EARTH,
            )
        };
        assert!(c(45.0).summer_k > c(-45.0).summer_k);
    }
}
