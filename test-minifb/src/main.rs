use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::time::Duration;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const BALL_RADIUS: usize = 100;
const BALL_COLOR: u32 = 0xFFFFCCFF; // Light yellow
const EYE_RADIUS: usize = 10;
const EYE_OFFSET_X: isize = 30;
const EYE_OFFSET_Y: isize = 20;
const EYE_COLOR: u32 = 0x000000FF; // Black
const MOUTH_WIDTH: usize = 60;
const MOUTH_HEIGHT: usize = 30;
const MOUTH_OFFSET_Y: isize = 40;
const MOUTH_COLOR: u32 = 0xFF0000FF; // Red

struct Ball {
    x: usize,
    y: usize,
    velocity_x: isize,
    velocity_y: isize,
}

impl Ball {
    fn new(x: usize, y: usize, velocity_x: isize, velocity_y: isize) -> Self {
        Ball {
            x,
            y,
            velocity_x,
            velocity_y,
        }
    }

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

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Human Face Ping Pong",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut ball = Ball::new(WIDTH / 2, HEIGHT / 2, 5, 5);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        ball.update();

        // Clear buffer
        buffer.iter_mut().for_each(|pixel| *pixel = 0);

        // Draw the ball (human face)
        draw_ball(&mut buffer, &ball);
        draw_eyes(&mut buffer, &ball);
        draw_mouth(&mut buffer, &ball);

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();

        thread::sleep(Duration::from_millis(16)); // Delay for 16 milliseconds
    }
}

fn draw_ball(buffer: &mut [u32], ball: &Ball) {
    draw_circle(buffer, ball.x, ball.y, BALL_RADIUS, BALL_COLOR);
}

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
