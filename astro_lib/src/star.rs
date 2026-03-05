use crate::error::{validate_mass, StarErr};

/// Luminosity in solar units: L = M^3.5
pub fn luminosity(mass: f64) -> Result<f64, StarErr> {
    validate_mass(mass)?;
    Ok(mass.powf(3.5))
}

/// Surface temperature in solar units: T = M^0.505
pub fn temperature(mass: f64) -> Result<f64, StarErr> {
    validate_mass(mass)?;
    Ok(mass.powf(0.505))
}

/// Radius from Stefan-Boltzmann in relative solar units: R = √L / T²
///
/// Derived from L = 4πR²σT⁴ — in solar units the constants cancel:
/// R_rel = √(L_rel) / T_rel²
pub fn radius(luminosity: f64, temperature: f64) -> f64 {
    luminosity.sqrt() / temperature.powi(2)
}

/// Star lifetime in solar units: τ = M^-2.5
pub fn lifetime(mass: f64) -> Result<f64, StarErr> {
    validate_mass(mass)?;
    Ok(mass.powf(-2.5))
}

#[derive(Debug, Clone, Copy)]
pub struct HabitableZone {
    /// Inner boundary in AU: 0.95 * √L
    pub inner: f64,
    /// Center (optimal) in AU: √L
    pub center: f64,
    /// Outer boundary in AU: 1.37 * √L
    pub outer: f64,
}

/// Habitable zone boundaries in AU
pub fn habitable_zone(luminosity: f64) -> HabitableZone {
    let center = luminosity.sqrt();
    HabitableZone {
        inner: 0.95 * center,
        center,
        outer: 1.37 * center,
    }
}

/// Frost line distance in AU: 4.85 * √L
pub fn frost_line(luminosity: f64) -> f64 {
    4.85 * luminosity.sqrt()
}

#[derive(Debug, Clone, Copy)]
pub struct SystemBoundaries {
    /// Inner boundary in AU: 0.1 * M
    pub inner: f64,
    /// Outer boundary in AU: 40 * M
    pub outer: f64,
}

/// Planetary system boundaries in AU
pub fn system_boundaries(mass: f64) -> Result<SystemBoundaries, StarErr> {
    validate_mass(mass)?;
    Ok(SystemBoundaries {
        inner: 0.1 * mass,
        outer: 40.0 * mass,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpectralClass {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

impl std::fmt::Display for SpectralClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SpectralClass::O => "O",
            SpectralClass::B => "B",
            SpectralClass::A => "A",
            SpectralClass::F => "F",
            SpectralClass::G => "G",
            SpectralClass::K => "K",
            SpectralClass::M => "M",
        };
        write!(f, "{s}")
    }
}

/// Spectral class derived from relative temperature (T_sun = 1.0, T_sun ≈ 5778 K)
pub fn spectral_class(temperature_rel: f64) -> SpectralClass {
    match temperature_rel {
        t if t > 5.190 => SpectralClass::O, // > 30 000 K
        t if t > 1.731 => SpectralClass::B, // 10 000–30 000 K
        t if t > 1.281 => SpectralClass::A, //  7 400–10 000 K
        t if t > 1.038 => SpectralClass::F, //  6 000–7 400 K
        t if t > 0.865 => SpectralClass::G, //  5 000–6 000 K
        t if t > 0.658 => SpectralClass::K, //  3 800–5 000 K
        _ => SpectralClass::M,              //  < 3 800 K
    }
}

/// Peak wavelength of stellar radiation in nm (Wien's law): λ = 501.5 / T_rel
///
/// 501.5 nm is the Sun's peak wavelength (T_sun ≈ 5778 K).
pub fn peak_wavelength(temperature_rel: f64) -> f64 {
    501.5 / temperature_rel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_has_unit_values() {
        let l = luminosity(1.0).unwrap();
        let t = temperature(1.0).unwrap();
        assert!((l - 1.0).abs() < 1e-10);
        assert!((t - 1.0).abs() < 1e-10);
        assert!((radius(l, t) - 1.0).abs() < 1e-10);
        assert!((lifetime(1.0).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn sun_spectral_class_is_g() {
        let t = temperature(1.0).unwrap();
        assert_eq!(spectral_class(t), SpectralClass::G);
    }

    #[test]
    fn sun_habitable_zone() {
        let l = luminosity(1.0).unwrap();
        let hz = habitable_zone(l);
        assert!((hz.center - 1.0).abs() < 1e-10);
        assert!((hz.inner - 0.95).abs() < 1e-10);
        assert!((hz.outer - 1.37).abs() < 1e-10);
    }

    #[test]
    fn sun_frost_line() {
        let l = luminosity(1.0).unwrap();
        assert!((frost_line(l) - 4.85).abs() < 1e-10);
    }

    /// Taounaris: M=1.03 → L≈1.11, T≈1.015 (from table)
    #[test]
    fn taounaris_values() {
        let l = luminosity(1.03).unwrap();
        let t = temperature(1.03).unwrap();
        assert!((l - 1.11).abs() < 0.01);
        assert!((t - 1.015).abs() < 0.005);
    }
}
