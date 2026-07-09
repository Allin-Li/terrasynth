use astro_lib::error::StarErr;
use astro_lib::planet::planet_radius_auto;
use astro_lib::star::{luminosity, radius as star_radius, temperature};
use leptos::prelude::*;

use super::storage::{
    ls_bool, ls_bool_dyn, ls_f64, ls_f64_dyn, ls_string_dyn, ls_u32_dyn, raw_get,
};

/// Planet host choices in a binary system.
pub const HOST_A: u32 = 0;
pub const HOST_B: u32 = 1;
pub const HOST_BOTH: u32 = 2;

/// Reactive inputs of one planet, persisted under `planet_{id}_*` keys.
#[derive(Clone, Copy)]
pub struct PlanetSigs {
    pub id: u32,
    pub name: RwSignal<String>,
    /// Which star the planet orbits (`HOST_A`/`HOST_B`/`HOST_BOTH`);
    /// only meaningful while binary mode is on.
    pub host: RwSignal<u32>,
    pub mass: RwSignal<f64>,
    pub use_manual_r: RwSignal<bool>,
    pub manual_radius: RwSignal<f64>,
    pub semi_major: RwSignal<f64>,
    pub eccentricity: RwSignal<f64>,
    pub axial_tilt: RwSignal<f64>,
    pub peri_long: RwSignal<f64>,
    pub albedo: RwSignal<f64>,
    pub co2_fraction: RwSignal<f64>,
    pub atmo_mass: RwSignal<f64>,
}

impl PlanetSigs {
    /// Planet 0 inherits values stored under the old single-planet keys
    /// (`planet_mass`, …), so worlds saved before multi-planet support keep
    /// their data on first load.
    pub fn new(id: u32) -> Self {
        let f = |suffix: &str, legacy: &str, def: f64| {
            let def = if id == 0 {
                raw_get(legacy).and_then(|v| v.parse().ok()).unwrap_or(def)
            } else {
                def
            };
            ls_f64_dyn(format!("planet_{id}_{suffix}"), def)
        };
        let b = |suffix: &str, legacy: &str, def: bool| {
            let def = if id == 0 {
                raw_get(legacy).and_then(|v| v.parse().ok()).unwrap_or(def)
            } else {
                def
            };
            ls_bool_dyn(format!("planet_{id}_{suffix}"), def)
        };

        Self {
            id,
            name: ls_string_dyn(format!("planet_{id}_name"), String::new()),
            host: ls_u32_dyn(format!("planet_{id}_host"), HOST_A),
            mass: f("mass", "planet_mass", 1.0),
            use_manual_r: b("use_manual_r", "planet_use_manual_r", false),
            manual_radius: f("manual_radius", "planet_manual_radius", 1.0),
            semi_major: f("semi_major", "planet_semi_major", 1.0),
            eccentricity: f("ecc", "planet_eccentricity", 0.017),
            axial_tilt: f("tilt", "planet_axial_tilt", 23.4),
            peri_long: f("peri_long", "planet_peri_long", 283.0),
            albedo: f("albedo", "planet_albedo", 0.3),
            co2_fraction: f("co2", "planet_co2_fraction", 0.0004),
            atmo_mass: f("atmo_mass", "planet_atmo_mass", 1.0),
        }
    }

    /// Effective radius: manual override or the mass-radius relation.
    pub fn eff_radius(&self) -> f64 {
        if self.use_manual_r.get() {
            self.manual_radius.get()
        } else {
            planet_radius_auto(self.mass.get())
        }
    }
}

/// What a planet orbits, photometrically and dynamically.
#[derive(Clone, Copy)]
pub struct StarParams {
    /// Mass driving Kepler orbits (total mass for circumbinary planets).
    pub kepler_mass: f64,
    /// Luminosity lighting the planet (combined for circumbinary).
    pub luminosity: f64,
    /// Effective temperature (relative). For a pair this is the primary's;
    /// together with the equivalent `radius` below it reproduces the
    /// combined luminosity in the equilibrium-temperature formula.
    pub temp: f64,
    pub radius: f64,
}

impl StarParams {
    pub const SUN: Self = Self { kepler_mass: 1.0, luminosity: 1.0, temp: 1.0, radius: 1.0 };

    pub fn single(mass: f64) -> Result<Self, StarErr> {
        let l = luminosity(mass)?;
        let t = temperature(mass)?;
        Ok(Self { kepler_mass: mass, luminosity: l, temp: t, radius: star_radius(l, t) })
    }

    /// Parameters for a planet with the given host in a binary of masses
    /// `ma`, `mb`. A custom override mass wins over everything.
    pub fn for_host(
        host: u32,
        binary: bool,
        custom: Option<f64>,
        ma: f64,
        mb: f64,
    ) -> Result<Self, StarErr> {
        if let Some(m) = custom {
            return Self::single(m);
        }
        if !binary {
            return Self::single(ma);
        }
        match host {
            HOST_B => Self::single(mb),
            HOST_BOTH => {
                let l = luminosity(ma)? + luminosity(mb)?;
                let t = temperature(ma)?;
                Ok(Self {
                    kepler_mass: ma + mb,
                    luminosity: l,
                    temp: t,
                    radius: star_radius(l, t),
                })
            }
            _ => Self::single(ma),
        }
    }
}

/// The star-configuration signals shared by every tab.
#[derive(Clone, Copy)]
pub struct StarInputs {
    pub mass_a: RwSignal<f64>,
    pub mass_b: RwSignal<f64>,
    pub binary: RwSignal<bool>,
    pub custom: RwSignal<bool>,
    pub custom_mass: RwSignal<f64>,
}

pub fn star_inputs() -> StarInputs {
    StarInputs {
        mass_a: ls_f64("star_mass", 1.0),
        mass_b: ls_f64("star_b_mass", 0.8),
        binary: ls_bool("star_binary_mode", false),
        custom: ls_bool("planet_custom_star", false),
        custom_mass: ls_f64("planet_custom_star_mass", 1.0),
    }
}

impl StarInputs {
    /// Parameters of what a planet with the given host setting orbits;
    /// falls back to the Sun on invalid masses.
    pub fn params_for(&self, host: u32) -> StarParams {
        let custom = self.custom.get().then(|| self.custom_mass.get());
        StarParams::for_host(host, self.binary.get(), custom, self.mass_a.get(), self.mass_b.get())
            .unwrap_or(StarParams::SUN)
    }
}

/// Short display tag for a planet's host star ("A", "B", "A + B").
pub fn host_tag(host: u32) -> &'static str {
    match host {
        HOST_B => "B",
        HOST_BOTH => "A + B",
        _ => "A",
    }
}
