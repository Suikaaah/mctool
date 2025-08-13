use std::time::{Duration, Instant};

pub struct Spam {
    is_active: bool,
    is_down: bool,
    last: Instant,
    interval: Duration,
    on_down: Box<dyn Fn()>,
    on_up: Box<dyn Fn()>,
}

impl Spam {
    pub fn new<F, G>(interval: Duration, on_down: F, on_up: G) -> Self
    where
        F: Fn() + 'static,
        G: Fn() + 'static,
    {
        Self {
            is_active: false,
            is_down: false,
            last: Instant::now(),
            interval,
            on_down: Box::new(on_down),
            on_up: Box::new(on_up),
        }
    }

    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    pub const fn _set_active(&mut self, value: bool) {
        self.is_active = value
    }

    pub const fn toggle_active(&mut self) {
        self.is_active ^= true
    }

    pub fn step(&mut self, now: Instant) {
        let elapsed = self.last - now;

        if self.interval <= elapsed {
            let rem = elapsed.as_secs_f64() % self.interval.as_secs_f64();
            self.last = now - Duration::from_secs_f64(rem);

            match (self.is_active, self.is_down) {
                (true, false) => {
                    (self.on_down)();
                    self.is_down = true;
                }
                (false, false) => (),
                (_, true) => {
                    (self.on_up)();
                    self.is_down = false;
                }
            }
        }
    }
}
