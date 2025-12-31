// random.rs
// Random number generator for the main thread and child threads

use std::sync::{Arc, Mutex};
use std::thread_local;
use rand::Rng;

// Thread-local random number generator
thread_local! {
    static RNG: Arc<Mutex<rand::rngs::ThreadRng>> = Arc::new(Mutex::new(rand::thread_rng()));
}

pub struct RandomUintGenerator {
    // For the Marsaglia algorithm
    rngx: u32,
    rngy: u32,
    rngz: u32,
    rngc: u32,
    // For the Jenkins algorithm
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl RandomUintGenerator {
    pub fn new() -> Self {
        RandomUintGenerator {
            rngx: 0,
            rngy: 0,
            rngz: 0,
            rngc: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
        }
    }

    pub fn initialize(&mut self, deterministic: bool, rng_seed: u32, thread_num: usize) {
        if deterministic {
            // Initialize Marsaglia deterministically per-thread
            self.rngx = rng_seed.wrapping_add(123456789).wrapping_add(thread_num as u32);
            self.rngy = rng_seed.wrapping_add(362436000).wrapping_add(thread_num as u32);
            self.rngz = rng_seed.wrapping_add(521288629).wrapping_add(thread_num as u32);
            self.rngc = rng_seed.wrapping_add(7654321).wrapping_add(thread_num as u32);
            if self.rngx == 0 {
                self.rngx = 123456789;
            }
            if self.rngy == 0 {
                self.rngy = 123456789;
            }
            if self.rngz == 0 {
                self.rngz = 123456789;
            }
            if self.rngc == 0 {
                self.rngc = 123456789;
            }

            // Initialize Jenkins deterministically per-thread
            self.a = 0xf1ea5eed;
            self.b = rng_seed.wrapping_add(thread_num as u32);
            self.c = self.b;
            self.d = self.b;
            if self.b == 0 {
                self.b = self.d.wrapping_add(123456789);
                self.c = self.b;
                self.d = self.b;
            }
        } else {
            // Non-deterministic initialization
            let mut rng = rand::thread_rng();

            // Initialize Marsaglia
            loop {
                self.rngx = rng.gen();
                if self.rngx != 0 {
                    break;
                }
            }
            loop {
                self.rngy = rng.gen();
                if self.rngy != 0 {
                    break;
                }
            }
            loop {
                self.rngz = rng.gen();
                if self.rngz != 0 {
                    break;
                }
            }
            loop {
                self.rngc = rng.gen();
                if self.rngc != 0 {
                    break;
                }
            }

            // Initialize Jenkins
            self.a = 0xf1ea5eed;
            loop {
                self.b = rng.gen();
                if self.b != 0 {
                    break;
                }
            }
            self.c = self.b;
            self.d = self.b;
        }
    }

    pub fn next(&mut self) -> u32 {
        // Jenkins algorithm (faster)
        fn rot32(x: u32, k: u32) -> u32 {
            (x << k) | (x >> (32 - k))
        }

        let e = self.a.wrapping_sub(rot32(self.b, 27));
        self.a = self.b ^ rot32(self.c, 17);
        self.b = self.c.wrapping_add(self.d);
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }

    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        assert!(max >= min);
        (self.next() % (max - min + 1)) + min
    }
}

impl Default for RandomUintGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Global thread-local random number generator
thread_local! {
    static RANDOM_UINT: std::cell::RefCell<RandomUintGenerator> = std::cell::RefCell::new(RandomUintGenerator::new());
}

pub fn random_uint() -> u32 {
    RANDOM_UINT.with(|rng| rng.borrow_mut().next())
}

pub fn random_uint_range(min: u32, max: u32) -> u32 {
    RANDOM_UINT.with(|rng| rng.borrow_mut().next_range(min, max))
}

pub fn initialize_random(deterministic: bool, rng_seed: u32, thread_num: usize) {
    RANDOM_UINT.with(|rng| {
        let mut r = rng.borrow_mut();
        r.initialize(deterministic, rng_seed, thread_num);
    });
}

pub const RANDOM_UINT_MAX: u32 = 0xffffffff;

