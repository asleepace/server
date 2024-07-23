use std::hash::{BuildHasher, Hasher, RandomState};

#[derive(Debug)]
pub struct Rand {
    random_state: RandomState,
    last_generated: u64,
}

impl Rand {
    pub fn new() -> Self {
        Rand {
            random_state: RandomState::new(),
            last_generated: 0,
        }
    }

    /**
        Generates a pseudo-random u64 value from the system time and the last generated value.
        NOTE: This is not cryptographically secure, but can be used for hashing.
    */
    pub fn generate_u64(&mut self) -> u64 {
        let mut hasher = RandomState::build_hasher(&self.random_state);
        let system_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        hasher.write_u64(system_time);
        let generated = hasher.finish();
        println!("[rand] generated hash: {:}", generated);
        self.last_generated = generated;
        generated
    }
}

/**
    This is a wrapper around the `Rand` struct to generate a random u64 value.
    Which creates a one-off instance of `Rand` and generates a random u64 value.
*/
pub fn generate_random_u64() -> u64 {
    Rand::new().generate_u64()
}
