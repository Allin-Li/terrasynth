use crate::star::HabitableZone;

/// Check if a planet's orbit falls within the star's habitable zone
pub fn is_in_habitable_zone(semi_major_axis_au: f64, hz: &HabitableZone) -> bool {
    semi_major_axis_au >= hz.inner && semi_major_axis_au <= hz.outer
}

/// Check if axial tilt is in the habitable range per Dole's criterion: 0–80° or 100–180°
pub fn is_habitable_tilt(axial_tilt_deg: f64) -> bool {
    (0.0..=80.0).contains(&axial_tilt_deg)
        || (100.0..=180.0).contains(&axial_tilt_deg)
}

/// Check if star mass is in the optimal range for habitability: 0.6–1.4 M☉
pub fn is_habitable_star_mass(mass: f64) -> bool {
    (0.6..=1.4).contains(&mass)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::star::{habitable_zone, luminosity};

    #[test]
    fn earth_is_in_solar_hz() {
        let l = luminosity(1.0).unwrap();
        let hz = habitable_zone(l);
        assert!(is_in_habitable_zone(1.0, &hz));
    }

    #[test]
    fn mercury_is_not_in_solar_hz() {
        let l = luminosity(1.0).unwrap();
        let hz = habitable_zone(l);
        assert!(!is_in_habitable_zone(0.39, &hz));
    }

    #[test]
    fn earth_tilt_is_habitable() {
        assert!(is_habitable_tilt(23.4));
    }

    #[test]
    fn tilt_above_80_is_not_habitable() {
        assert!(!is_habitable_tilt(85.0));
    }

    #[test]
    fn sun_mass_is_habitable() {
        assert!(is_habitable_star_mass(1.0));
    }
}
