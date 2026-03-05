use crate::star::SpectralClass;

/// Predicted dominant pigment color for photosynthetic organisms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloraPigment {
    /// Absorbs all available light (very dim stars)
    Black,
    /// Absorbs red + blue, reflects green (Sun-like)
    Green,
    /// Absorbs blue + green, reflects yellow-orange
    Orange,
    /// Absorbs UV/blue/green, reflects red
    Red,
    /// Absorbs orange/red/green, reflects blue
    Blue,
    /// Too much UV for surface flora
    UvHostile,
}

impl std::fmt::Display for FloraPigment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FloraPigment::Black => write!(f, "Black"),
            FloraPigment::Green => write!(f, "Green"),
            FloraPigment::Orange => write!(f, "Orange"),
            FloraPigment::Red => write!(f, "Red"),
            FloraPigment::Blue => write!(f, "Blue"),
            FloraPigment::UvHostile => write!(f, "UV-hostile"),
        }
    }
}

/// Detailed flora pigment prediction with reasoning.
pub struct FloraPrediction {
    pub pigment: FloraPigment,
    pub peak_wavelength_nm: f64,
    pub absorbed_range: &'static str,
    pub reflected_color: &'static str,
    pub reasoning: &'static str,
}

/// Predict flora pigment color from star spectral class and peak wavelength.
///
/// Based on Kiang et al. (2007) "Spectral Signatures of Photosynthesis".
pub fn predict_flora_pigment(spectral: SpectralClass, peak_wavelength_nm: f64) -> FloraPrediction {
    match spectral {
        SpectralClass::O | SpectralClass::B => FloraPrediction {
            pigment: FloraPigment::UvHostile,
            peak_wavelength_nm,
            absorbed_range: "N/A",
            reflected_color: "N/A",
            reasoning: "Star peaks in extreme UV. Surface flora is unlikely without \
                       a substantial ozone layer. Underwater photosynthesis may use \
                       scattered visible light.",
        },
        SpectralClass::A => FloraPrediction {
            pigment: FloraPigment::Red,
            peak_wavelength_nm,
            absorbed_range: "UV, blue, green (300-550 nm)",
            reflected_color: "Red / orange",
            reasoning: "A-type star peaks in UV/blue. Flora would absorb the abundant \
                       blue and green photons, reflecting longer wavelengths. Plants \
                       would appear red or deep orange.",
        },
        SpectralClass::F => FloraPrediction {
            pigment: FloraPigment::Orange,
            peak_wavelength_nm,
            absorbed_range: "Blue, green (400-550 nm)",
            reflected_color: "Yellow-orange",
            reasoning: "F-type star peaks in blue. Flora absorbs the dominant blue \
                       and green wavelengths, reflecting yellow-orange. Plants would \
                       appear orange or golden.",
        },
        SpectralClass::G => FloraPrediction {
            pigment: FloraPigment::Green,
            peak_wavelength_nm,
            absorbed_range: "Red (600-700 nm) + Blue (400-500 nm)",
            reflected_color: "Green",
            reasoning: "G-type star (like our Sun) peaks near green-yellow. Flora \
                       absorbs red and blue photons (most energy-efficient) while \
                       reflecting green. This is why Earth plants are green.",
        },
        SpectralClass::K => FloraPrediction {
            pigment: FloraPigment::Blue,
            peak_wavelength_nm,
            absorbed_range: "Orange, red, green (500-750 nm)",
            reflected_color: "Blue-violet",
            reasoning: "K-type star peaks in orange-red. Flora absorbs the dominant \
                       orange/red photons and most visible light, reflecting the \
                       least useful blue wavelengths. Plants would appear blue-purple.",
        },
        SpectralClass::M => FloraPrediction {
            pigment: FloraPigment::Black,
            peak_wavelength_nm,
            absorbed_range: "All visible + near-IR (400-1000 nm)",
            reflected_color: "Very dark / black",
            reasoning: "M-dwarf star is dim and peaks in infrared. Flora must absorb \
                       every available photon across all visible wavelengths to gather \
                       enough energy. Plants would appear black or very dark purple.",
        },
    }
}

/// Display hex color for the predicted pigment (for UI swatches).
pub fn pigment_display_color(pigment: FloraPigment) -> &'static str {
    match pigment {
        FloraPigment::Black => "#1a1a2e",
        FloraPigment::Green => "#2ecc71",
        FloraPigment::Orange => "#e67e22",
        FloraPigment::Red => "#e74c3c",
        FloraPigment::Blue => "#3498db",
        FloraPigment::UvHostile => "#7f8c8d",
    }
}
