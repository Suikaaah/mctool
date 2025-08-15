mod coord;
mod engine;
mod font_engine;
mod grid;
mod io;
mod state;

fn main() {
    use {engine::Engine, font_engine::FontEngine, state::State};

    let mut engine = Engine::new();
    let mut state = State::new();
    let font_engine = FontEngine::new();
    let font = font_engine.load_font(24).expect("failed to load font");

    'main_loop: loop {
        while let Some(event) = engine.poll_event() {
            if let sdl2::event::Event::Quit { .. } = event {
                break 'main_loop;
            }
        }

        state.step();
        engine.draw(&state, &font);
        Engine::sleep();
    }
}
