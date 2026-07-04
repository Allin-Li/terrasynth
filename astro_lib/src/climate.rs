use std::f64::consts::PI;

/// Fraction of the equator-pole radiative imbalance smoothed out by
/// atmospheric/oceanic heat transport. 0 = pure radiative balance,
/// 1 = perfectly mixed. Calibrated so Earth gives ≈ +25 °C at the
/// equator and ≈ −15 °C at the poles.
pub const HEAT_TRANSPORT_EARTH: f64 = 0.3;

/// How much of the instantaneous solstice radiative swing actually shows
/// up as seasonal temperature change (thermal inertia damping).
/// Calibrated so Earth gets a boreal belt at 60° and ≈ −30 °C Arctic winters.
pub const SEASONAL_DAMPING_EARTH: f64 = 0.35;

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

/// Annual mean insolation at a latitude, as a fraction of the solar
/// constant (circular orbit; the global mean is exactly 1/4).
pub fn annual_mean_insolation_rel(latitude_deg: f64, axial_tilt_deg: f64) -> f64 {
    const STEPS: usize = 144;
    let mut sum = 0.0;
    for i in 0..STEPS {
        let lambda = 2.0 * PI * (i as f64 + 0.5) / STEPS as f64;
        let decl = solar_declination_deg(axial_tilt_deg, lambda);
        sum += daily_insolation_rel(latitude_deg, decl);
    }
    sum / STEPS as f64
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
    pub latitude_deg: f64,
    /// Annual mean surface temperature, K
    pub annual_k: f64,
    /// Warm-season (summer solstice) temperature, K
    pub summer_k: f64,
    /// Cold-season (winter solstice) temperature, K
    pub winter_k: f64,
    pub zone: ClimateZone,
}

/// Radiative-equilibrium temperature for a given insolation ratio,
/// smoothed by heat transport: T = T_mean · ((1−f)·I/Ī + f)^¼.
fn balance_temp_k(mean_temp_k: f64, insolation_ratio: f64, heat_transport: f64) -> f64 {
    mean_temp_k * ((1.0 - heat_transport) * insolation_ratio + heat_transport).powf(0.25)
}

/// Energy-balance climate for one latitude.
///
/// - `mean_surface_temp_k`: the planet's global mean surface temperature
///   (equilibrium + greenhouse), which this model redistributes.
/// - `heat_transport`: 0–1, see [`HEAT_TRANSPORT_EARTH`].
/// - `seasonal_damping`: 0–1, see [`SEASONAL_DAMPING_EARTH`].
///
/// Assumes a circular orbit and a rotating planet (not tidally locked).
pub fn latitude_climate(
    latitude_deg: f64,
    axial_tilt_deg: f64,
    mean_surface_temp_k: f64,
    heat_transport: f64,
    seasonal_damping: f64,
) -> LatitudeClimate {
    const GLOBAL_MEAN_INSOLATION: f64 = 0.25;
    let annual_ratio =
        annual_mean_insolation_rel(latitude_deg, axial_tilt_deg) / GLOBAL_MEAN_INSOLATION;
    let annual_k = balance_temp_k(mean_surface_temp_k, annual_ratio, heat_transport);

    // Instantaneous radiative response at the solstices, damped by
    // thermal inertia toward the annual mean.
    let tilt_eff = axial_tilt_deg.to_radians().sin().asin().to_degrees().abs();
    let season = |decl: f64| {
        let ratio = daily_insolation_rel(latitude_deg, decl) / GLOBAL_MEAN_INSOLATION;
        let instant = balance_temp_k(mean_surface_temp_k, ratio, heat_transport);
        annual_k + seasonal_damping * (instant - annual_k)
    };
    let t_a = season(tilt_eff);
    let t_b = season(-tilt_eff);
    let summer_k = t_a.max(t_b);
    let winter_k = t_a.min(t_b);

    LatitudeClimate {
        latitude_deg,
        annual_k,
        summer_k,
        winter_k,
        zone: climate_zone(summer_k - 273.15, winter_k - 273.15),
    }
}

/// Climate for the standard 0–90° latitude bands (15° step) with
/// Earth-calibrated transport and damping.
pub fn climate_bands(axial_tilt_deg: f64, mean_surface_temp_k: f64) -> Vec<LatitudeClimate> {
    (0..=6)
        .map(|i| {
            latitude_climate(
                (i * 15) as f64,
                axial_tilt_deg,
                mean_surface_temp_k,
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
        latitude_climate(lat, EARTH_TILT, EARTH_MEAN_K, HEAT_TRANSPORT_EARTH, SEASONAL_DAMPING_EARTH)
    }

    #[test]
    fn global_mean_insolation_is_quarter() {
        // cos-weighted average over latitudes must equal S0/4
        let (mut sum, mut w) = (0.0, 0.0);
        for i in 0..90 {
            let lat = i as f64 + 0.5;
            let weight = lat.to_radians().cos();
            sum += annual_mean_insolation_rel(lat, EARTH_TILT) * weight;
            w += weight;
        }
        assert!((sum / w - 0.25).abs() < 0.003);
    }

    #[test]
    fn earth_equator_annual_insolation() {
        // Observed TOA annual mean at the equator ≈ 0.30 S0
        let i = annual_mean_insolation_rel(0.0, EARTH_TILT);
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
        let c = latitude_climate(45.0, 0.0, EARTH_MEAN_K, HEAT_TRANSPORT_EARTH, SEASONAL_DAMPING_EARTH);
        assert!((c.summer_k - c.winter_k).abs() < 0.1);
    }

    #[test]
    fn extreme_tilt_pole_beats_equator() {
        // Above ~54° tilt the poles receive more annual energy than the equator
        let pole = annual_mean_insolation_rel(90.0, 90.0);
        let equator = annual_mean_insolation_rel(0.0, 90.0);
        assert!(pole > equator);
    }

    #[test]
    fn retrograde_tilt_folds() {
        // 177° retrograde tilt behaves like 3°
        let a = annual_mean_insolation_rel(45.0, 177.0);
        let b = annual_mean_insolation_rel(45.0, 3.0);
        assert!((a - b).abs() < 1e-9);
    }
}
