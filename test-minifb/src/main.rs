use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;
use std::thread;
use std::time::Duration;

// Set the dimensions of the window
const WIDTH: usize = 640;
const HEIGHT: usize = 480;

// Constants for the ball (human face)
const BALL_RADIUS: usize = 100;
const BALL_COLOR: u32 = 0xFFFFCCFF; // Light yellow

// Constants for the eyes
const EYE_RADIUS: usize = 10;
const EYE_OFFSET_X: isize = 30;
const EYE_OFFSET_Y: isize = 20;
const EYE_COLOR: u32 = 0x000000FF; // Black

// Constants for the mouth
const MOUTH_WIDTH: usize = 60;
const MOUTH_HEIGHT: usize = 30;
const MOUTH_OFFSET_Y: isize = 40;
const MOUTH_COLOR: u32 = 0xFF0000FF; // Red

// Constants for the particles
const PARTICLE_COUNT: usize = 1000;
const GRAVITY: f32 = 0.2;

// Structure for the ball (human face)
struct Ball {
    x: isize,
    y: isize,
    velocity_x: isize,
    velocity_y: isize,
}

impl Ball {
    // Create a new ball
    fn new(x: isize, y: isize, velocity_x: isize, velocity_y: isize) -> Self {
        Ball {
            x,
            y,
            velocity_x,
            velocity_y,
        }
    }

    // Update the position of the ball
    fn update(&mut self, mouse_x: isize, mouse_y: isize, window_width: usize, window_height: usize) {
        if mouse_x >= 0 && mouse_x < window_width as isize && mouse_y >= 0 && mouse_y < window_height as isize {
            self.velocity_x = (mouse_x - self.x).signum() * 5;
            self.velocity_y = (mouse_y - self.y).signum() * 5;
        }

        let new_x = self.x + self.velocity_x;
        let new_y = self.y + self.velocity_y;

        // Check if the new position is within the valid range
        if new_x + BALL_RADIUS as isize >= window_width as isize || new_x < BALL_RADIUS as isize {
            self.velocity_x = -self.velocity_x;
        }
        if new_y + BALL_RADIUS as isize >= window_height as isize || new_y < BALL_RADIUS as isize {
            self.velocity_y = -self.velocity_y;
        }

        self.x = new_x;
        self.y = new_y;
    }
}

// Structure for the particles
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: u32,
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Human Face Mov Mov",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut ball = Ball::new(WIDTH as isize / 2, HEIGHT as isize / 2, 5, 5);
    let mut particles: Vec<Particle> = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            ball.update(
                mouse_x as isize,
                mouse_y as isize,
                WIDTH,
                HEIGHT,
            );
        } else {
            ball.update(
                ball.x,
                ball.y,
                WIDTH,
                HEIGHT,
            );
        }

        // Clear the buffer
        buffer.iter_mut().for_each(|pixel| *pixel = 0);

        // Spawn fireworks randomly
        if thread_rng().gen::<f32>() < 0.02 {
            spawn_firework(&mut particles, ball.x as usize, ball.y as usize);
        }

        // Update and draw the particles
        update_particles(&mut particles);
        draw_particles(&mut buffer, &particles);

        // Draw the ball (human face)
        draw_ball(&mut buffer, &ball);
        draw_eyes(&mut buffer, &ball);
        draw_mouth(&mut buffer, &ball);

        // Update the window
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();

        // Delay for a short period
        thread::sleep(Duration::from_millis(16));
    }
}

// Draw the ball (human face)
fn draw_ball(buffer: &mut [u32], ball: &Ball) {
    draw_circle(buffer, ball.x as usize, ball.y as usize, BALL_RADIUS, BALL_COLOR);
}

// Draw the eyes
fn draw_eyes(buffer: &mut [u32], ball: &Ball) {
    draw_circle(
        buffer,
        (ball.x - EYE_OFFSET_X) as usize,
        (ball.y - EYE_OFFSET_Y) as usize,
        EYE_RADIUS,
        EYE_COLOR,
    );
    draw_circle(
        buffer,
        (ball.x + EYE_OFFSET_X) as usize,
        (ball.y - EYE_OFFSET_Y) as usize,
        EYE_RADIUS,
        EYE_COLOR,
    );
}

// Draw the mouth
fn draw_mouth(buffer: &mut [u32], ball: &Ball) {
    let mouth_x = ball.x - MOUTH_WIDTH as isize / 2;
    let mouth_y = ball.y + MOUTH_OFFSET_Y as isize;
    draw_rectangle(
        buffer,
        mouth_x as usize,
        mouth_y as usize,
        MOUTH_WIDTH,
        MOUTH_HEIGHT,
        MOUTH_COLOR,
    );
}

// Draw a circle on the buffer
fn draw_circle(buffer: &mut [u32], center_x: usize, center_y: usize, radius: usize, color: u32) {
    let start_y = center_y.saturating_sub(radius);
    let end_y = center_y.saturating_add(radius);
    let start_x = center_x.saturating_sub(radius);
    let end_x = center_x.saturating_add(radius);

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            if x < WIDTH && y < HEIGHT {
                let dx = x as isize - center_x as isize;
                let dy = y as isize - center_y as isize;
                let distance_squared = (dx * dx + dy * dy) as usize;
                if distance_squared <= radius * radius {
                    buffer[(y * WIDTH) + x] = color;
                }
            }
        }
    }
}

// Draw a rectangle on the buffer
fn draw_rectangle(buffer: &mut [u32], x: usize, y: usize, width: usize, height: usize, color: u32) {
    let end_x = x + width;
    let end_y = y + height;

    for dy in y..end_y {
        for dx in x..end_x {
            if dx < WIDTH && dy < HEIGHT {
                buffer[(dy * WIDTH) + dx] = color;
            }
        }
    }
}

// Spawn fireworks particles
fn spawn_firework(particles: &mut Vec<Particle>, x: usize, y: usize) {
    for _ in 0..PARTICLE_COUNT {
        let angle = thread_rng().gen_range(0.0..2.0 * std::f32::consts::PI);
        let speed = thread_rng().gen_range(1.0..5.0);
        let color = generate_random_color();

        let vx = speed * angle.cos();
        let vy = speed * angle.sin();

        let particle = Particle {
            x: x as f32,
            y: y as f32,
            vx,
            vy,
            color,
        };

        particles.push(particle);
    }
}

// Update the particles' positions
fn update_particles(particles: &mut Vec<Particle>) {
    particles.iter_mut().for_each(|particle| {
        particle.vy += GRAVITY;
        particle.x += particle.vx;
        particle.y += particle.vy;
    });

    particles.retain(|particle| particle.y < HEIGHT as f32);
}

// Draw the particles on the buffer
fn draw_particles(buffer: &mut [u32], particles: &[Particle]) {
    particles.iter().for_each(|particle| {
        let x = particle.x as usize;
        let y = particle.y as usize;

        if x < WIDTH && y < HEIGHT {
            buffer[(y * WIDTH) + x] = particle.color;
        }
    });
}

// Generate a random color
fn generate_random_color() -> u32 {
    let r = thread_rng().gen_range(0..=255);
    let g = thread_rng().gen_range(0..=255);
    let b = thread_rng().gen_range(0..=255);
    let a = thread_rng().gen_range(0..=255);

    (a << 24) | (r << 16) | (g << 8) | b
}
