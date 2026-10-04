// A face that follows your mouse, with fireworks, in a real window.
//
// Every frame of every game works the same way:
//   1. read the input (where is the mouse? is Esc pressed?)
//   2. update the world (move the face and the particles)
//   3. draw the world into a buffer of pixels
//   4. show the buffer, and wait for the next frame
//
// All of steps 2 and 3 are in scene.rs, with no window, so they're testable.

mod scene;

use minifb::{Key, MouseMode, Window, WindowOptions};
use scene::{Face, HEIGHT, Random, WIDTH};

fn main() {
    let mut window = match Window::new("Bouncing face: move the mouse, Esc to quit", WIDTH, HEIGHT, WindowOptions::default()) {
        Ok(window) => window,
        Err(error) => {
            // For example on a server without a display: say so, don't panic.
            eprintln!("can't open a window: {error}");
            std::process::exit(1);
        }
    };
    window.set_target_fps(60); // minifb waits between frames for us

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    let mut face = Face::new();
    let mut particles = Vec::new();
    let mut random = Random::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // 1. input: `Discard` gives None when the mouse is outside the window
        let mouse = window.get_mouse_pos(MouseMode::Discard);

        // 2. update
        face.update(mouse);
        if random.unit() < 0.02 {
            scene::spawn_firework(&mut particles, face.x, face.y, &mut random);
        }
        scene::update_particles(&mut particles);

        // 3. draw, 4. show
        scene::draw(&mut buffer, &face, &particles);
        if let Err(error) = window.update_with_buffer(&buffer, WIDTH, HEIGHT) {
            eprintln!("can't update the window: {error}");
            break;
        }
    }
}
