use core::types::Float;
use rand::random;

pub struct RNG {}

impl RNG {
    pub fn new() -> RNG {
        RNG {}
    }

    pub fn from_seed(seed: u32) -> RNG {
        RNG::new() // TODO: use seed
    }

    pub fn random_float(&mut self) -> Float {
        random()
    }
}
