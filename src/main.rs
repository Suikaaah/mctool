mod engine;
mod io;
mod state;

use {engine::Engine, state::State};

fn main() {
    let mut engine = Engine::new();
    let mut state = State::new();

    'main_loop: loop {
        while let Some(event) = engine.poll_event() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main_loop;
            }
        }

        state.step();
        engine.draw(&state);
        Engine::sleep();
    }
}
