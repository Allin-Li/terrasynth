use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum StarErr {
    #[error("mass must not be zero")]
    ZeroMass,
    #[error("mass must not be NaN")]
    NanMass,
    #[error("mass must not be negative")]
    NegativeMass,
}

pub(crate) fn validate_mass(mass: f64) -> Result<(), StarErr> {
    if mass.is_nan() {
        return Err(StarErr::NanMass);
    }
    if mass == 0.0 {
        return Err(StarErr::ZeroMass);
    }
    if mass < 0.0 {
        return Err(StarErr::NegativeMass);
    }
    Ok(())
}
