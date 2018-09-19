use core::types::Float;
use rand::random;

pub struct RNG {}

impl RNG {
    pub fn new() -> RNG {
        RNG {}
    }

    pub fn random_float(&mut self) -> Float {
        random()
    }
}
