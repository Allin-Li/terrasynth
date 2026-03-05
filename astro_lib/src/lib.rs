pub mod atmosphere;
pub mod binary;
pub mod error;
pub mod flora;
pub mod habitability;
pub mod moon;
pub mod orbit;
pub mod planet;
pub mod star;

pub use atmosphere::{
    atmosphere_retention, can_retain_gas, equilibrium_temperature, escape_velocity_ms,
    exosphere_temperature_estimate, greenhouse_effect, jeans_parameter, partial_pressure,
    scale_height, surface_pressure_estimate, surface_temperature, thermal_velocity,
    AtmosphereRetention, GasMasses,
};
pub use binary::{
    binary_habitable_zone, binary_orbital_period, combined_luminosity, is_orbit_stable_binary,
    p_type_critical_radius, s_type_critical_radius, BinaryOrbitType,
};
pub use error::StarErr;
pub use habitability::{is_habitable_star_mass, is_habitable_tilt, is_in_habitable_zone};
pub use moon::{
    angular_size_arcmin, are_moons_stable, hill_sphere_au, hill_sphere_planet_radii,
    is_moon_orbit_valid, moon_gravity, moon_mass, moon_orbital_period_days, near_resonance,
    roche_limit_planet_radii, stable_orbit_limit,
};
pub use orbit::{
    aphelion, eccentricity_from_n_planets, orbital_period, orbital_velocity, perihelion,
    polar_circle, tropic_latitude, OrbitalPeriod,
};
pub use planet::{
    density, escape_velocity, gravity, has_solid_surface, planet_radius_auto,
    planet_radius_from_mass, planet_type, surface_area, volume, PlanetType,
};
pub use flora::{pigment_display_color, predict_flora_pigment, FloraPigment, FloraPrediction};
pub use star::{
    frost_line, habitable_zone, lifetime, luminosity, peak_wavelength, radius, spectral_class,
    system_boundaries, temperature, HabitableZone, SpectralClass, SystemBoundaries,
};
