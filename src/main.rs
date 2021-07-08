extern crate sdl;
mod chip;

use sdl::event::Event;
use std::io;

fn main() {
    let mut cpu = chip::cpu::Cpu::new();

    let gamefile = {
        println!("Give the name of the game that you want to load:");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        format!("rom/{}", input.trim_end())
    };

    cpu.load(gamefile);
    sdl::init(&[
        sdl::InitFlag::Video,
        sdl::InitFlag::Audio,
        sdl::InitFlag::Timer,
    ]);

    'main: loop {
        'event: loop {
            match sdl::event::poll_event() {
                Event::Quit => break 'main,
                Event::None => break 'event,
                Event::Key(key, state, _, _) => cpu.press(key, state),
                _ => {}
            }
        }

        cpu.tick();
        cpu.draw();
    }

    sdl::quit();
}
