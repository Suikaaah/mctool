use crate::State;
use sdl2::{EventPump, Sdl, event::Event, render::Canvas, video::Window};
use std::time::Duration;

pub struct Engine {
    event_pump: EventPump,
    canvas: Canvas<Window>,
    _context: Sdl,
}

impl Engine {
    const TITLE: &str = "mctool";
    const WIDTH: u32 = 500;
    const HEIGHT: u32 = 300;
    const POLLING_RATE: Duration = Duration::from_millis(2);

    pub fn new() -> Self {
        let context = sdl2::init().expect("failed to initialize sdl2");

        let canvas = context
            .video()
            .expect("failed to initialize video")
            .window(Self::TITLE, Self::WIDTH, Self::HEIGHT)
            .position_centered()
            .build()
            .expect("failed to build window")
            .into_canvas()
            .accelerated()
            .build()
            .expect("failed to build canvas");

        let event_pump = context.event_pump().expect("failed to obtain event_pump");

        Self {
            event_pump,
            canvas,
            _context: context,
        }
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.event_pump.poll_event()
    }

    pub fn draw(&mut self, state: &State) {
        if state.is_modified {
            if state.spam_left.is_active() {
                println!("left");
            }

            if state.spam_right.is_active() {
                println!("right");
            }

            if state.spam_space.is_active() {
                println!("space");
            }

            self.canvas.clear();
            self.canvas.present();

            println!("drawn");
        }
    }

    pub fn sleep() {
        std::thread::sleep(Self::POLLING_RATE);
    }
}
