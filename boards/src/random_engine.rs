use rand::RngCore;

pub trait RandomEngine {
    fn next(&mut self) -> u64;
}

pub struct DefaultRandomEngine;

impl DefaultRandomEngine {
    pub fn new() -> Self { return Self }
}

impl RandomEngine for DefaultRandomEngine {
    fn next(&mut self) -> u64 {
        rand::random()
    }
}

struct RngCoreWrapper<'a, T>(&'a mut T);

impl<'a, T> RngCore for RngCoreWrapper<'a, T>
where
    T: RandomEngine,
{
    fn next_u32(&mut self) -> u32 {
        self.0.next() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next()
    }

    fn fill_bytes(&mut self, _: &mut [u8]) {
        todo!()
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> std::result::Result<(), rand::Error> {
        todo!()
    }
}

pub fn to_rng_core<'a, T: RandomEngine>(r: &'a mut T) -> impl RngCore + 'a {
    RngCoreWrapper(r)
}

impl RandomEngine for dyn FnMut() -> u64 {
    fn next(&mut self) -> u64 {
        self()
    }
}

pub fn default() -> DefaultRandomEngine {
    DefaultRandomEngine {}
}

pub struct XorShifEngine {
    state: u64,
}

impl XorShifEngine {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl RandomEngine for XorShifEngine {
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        self.state
    }
}
