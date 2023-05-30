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
    x: usize,
    y: usize,
    velocity_x: isize,
    velocity_y: isize,
}

impl Ball {
    // Create a new ball
    fn new(x: usize, y: usize, velocity_x: isize, velocity_y: isize) -> Self {
        Ball {
            x,
            y,
            velocity_x,
            velocity_y,
        }
    }

    // Update the position of the ball
    fn update(&mut self) {
        let new_x = (self.x as isize).wrapping_add(self.velocity_x) as usize;
        let new_y = (self.y as isize).wrapping_add(self.velocity_y) as usize;

        // Check if the new position is within the valid range
        if new_x + BALL_RADIUS >= WIDTH || new_x < BALL_RADIUS {
            self.velocity_x = -self.velocity_x;
        }
        if new_y + BALL_RADIUS >= HEIGHT || new_y < BALL_RADIUS {
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
        "Human Face Mouse Move",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut ball = Ball::new(WIDTH / 2, HEIGHT / 2, 5, 5);
    let mut particles: Vec<Particle> = Vec::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        ball.update();

        // Clear the buffer
        buffer.iter_mut().for_each(|pixel| *pixel = 0);

        // Spawn fireworks randomly
        if thread_rng().gen::<f32>() < 0.02 {
            spawn_firework(&mut particles, ball.x, ball.y);
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

        // Get the mouse position (handling when mouse is outside window)
        let (mouse_x, mouse_y) = if let Some(pos) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            pos
        } else {
            (ball.x as f32, ball.y as f32)
        };

        // Calculate the velocity based on the mouse position
        let velocity_x = (mouse_x as isize - ball.x as isize) / 30;
        let velocity_y = (mouse_y as isize - ball.y as isize) / 30;

        // Update the ball velocity
        ball.velocity_x = velocity_x;
        ball.velocity_y = velocity_y;

        // Delay for a short period
        thread::sleep(Duration::from_millis(16));
    }
}

// Draw the ball (human face)
fn draw_ball(buffer: &mut [u32], ball: &Ball) {
    draw_circle(buffer, ball.x, ball.y, BALL_RADIUS, BALL_COLOR);
}

// Draw the eyes
fn draw_eyes(buffer: &mut [u32], ball: &Ball) {
    draw_circle(
        buffer,
        ball.x - EYE_OFFSET_X as usize,
        ball.y - EYE_OFFSET_Y as usize,
        EYE_RADIUS,
        EYE_COLOR,
    );
    draw_circle(
        buffer,
        ball.x + EYE_OFFSET_X as usize,
        ball.y - EYE_OFFSET_Y as usize,
        EYE_RADIUS,
        EYE_COLOR,
    );
}

// Draw the mouth
fn draw_mouth(buffer: &mut [u32], ball: &Ball) {
    let mouth_x = ball.x - MOUTH_WIDTH / 2;
    let mouth_y = ball.y + MOUTH_OFFSET_Y as usize;
    draw_rectangle(
        buffer,
        mouth_x,
        mouth_y,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ball_update_within_bounds() {
        let mut ball = Ball::new(WIDTH / 2, HEIGHT / 2, 5, 5);

        ball.update();

        assert!(ball.x >= BALL_RADIUS && ball.x <= WIDTH - BALL_RADIUS);
        assert!(ball.y >= BALL_RADIUS && ball.y <= HEIGHT - BALL_RADIUS);
    }

    #[test]
    fn test_ball_update_hit_left_wall() {
        let mut ball = Ball::new(BALL_RADIUS, HEIGHT / 2, -5, 0);

        ball.update();

        assert_eq!(ball.velocity_x, 5);
    }

    #[test]
    fn test_ball_update_hit_top_wall() {
        let mut ball = Ball::new(WIDTH / 2, BALL_RADIUS, 0, -5);

        ball.update();

        assert_eq!(ball.velocity_y, 5);
    }

    #[test]
    fn test_ball_update_hit_bottom_wall() {
        let mut ball = Ball::new(WIDTH / 2, HEIGHT - BALL_RADIUS, 0, 5);

        ball.update();

        assert_eq!(ball.velocity_y, -5);
    }

    #[test]
    fn test_ball_update_hit_right_wall() {
        let mut ball = Ball::new(WIDTH - BALL_RADIUS, HEIGHT / 2, 5, 0);

        ball.update();

        assert_eq!(ball.velocity_x, -5);
    }
}
