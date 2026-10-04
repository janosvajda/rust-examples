//! Everything that moves and gets drawn, with no window involved, so all of
//! it can be tested. A picture is just a `Vec<u32>`: one number per pixel.

pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;

/// Colours are `0x00RRGGBB`: red, green and blue, 0–255 each. The top byte is
/// unused, so `0xFFFFCCFF` would NOT be light yellow with full opacity: it's
/// red FF, green CC, blue FF, which is lavender.
pub const FACE_COLOUR: u32 = 0x00FF_E066; // warm yellow
pub const EYE_COLOUR: u32 = 0x0020_2020; // almost black
pub const MOUTH_COLOUR: u32 = 0x00D0_3030; // red
pub const BACKGROUND: u32 = 0x0010_1828; // dark blue

pub const FACE_RADIUS: f32 = 100.0;
const EYE_RADIUS: f32 = 10.0;
const EYE_OFFSET: (f32, f32) = (30.0, -20.0);
const MOUTH_SIZE: (f32, f32) = (60.0, 14.0);
const MOUTH_OFFSET_Y: f32 = 40.0;

const PARTICLES_PER_FIREWORK: usize = 300;
/// A hard limit, so the particle list can never grow without bound.
pub const MAX_PARTICLES: usize = 5_000;
const GRAVITY: f32 = 0.15;

/// The face: a position and a velocity, in pixels and pixels per frame.
/// Floats, so it can move smoothly by fractions of a pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
}

impl Face {
    pub fn new() -> Self {
        Face { x: WIDTH as f32 / 2.0, y: HEIGHT as f32 / 2.0, vx: 3.0, vy: 2.0 }
    }

    /// One frame. With the mouse in the window, steer towards it; without,
    /// keep moving and bounce off the walls. Either way, the face always stays
    /// completely inside the window.
    pub fn update(&mut self, mouse: Option<(f32, f32)>) {
        if let Some((mx, my)) = mouse {
            // Move a fraction of the remaining distance: fast when far, gentle when close.
            self.vx = (mx - self.x) / 12.0;
            self.vy = (my - self.y) / 12.0;
        } else if self.vx.abs() + self.vy.abs() < 1.0 {
            (self.vx, self.vy) = (3.0, 2.0); // the mouse just left: start bouncing again
        }
        self.x += self.vx;
        self.y += self.vy;

        // Bounce: reverse the direction AND put the face back inside the window.
        let (min_x, max_x) = (FACE_RADIUS, WIDTH as f32 - FACE_RADIUS);
        let (min_y, max_y) = (FACE_RADIUS, HEIGHT as f32 - FACE_RADIUS);
        if self.x < min_x || self.x > max_x {
            self.vx = -self.vx;
            self.x = self.x.clamp(min_x, max_x);
        }
        if self.y < min_y || self.y > max_y {
            self.vy = -self.vy;
            self.y = self.y.clamp(min_y, max_y);
        }
    }
}

impl Default for Face {
    fn default() -> Self {
        Face::new()
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    colour: u32,
}

/// A tiny pseudo-random number generator (xorshift). Plenty for fireworks,
/// and no crate needed. Never use something like this for security.
pub struct Random(u64);

impl Random {
    /// Seeded from the standard library's per-process random hash keys.
    pub fn new() -> Self {
        use std::hash::{BuildHasher, RandomState};
        Random(RandomState::new().hash_one(0u8) | 1)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }

    /// A float from 0.0 up to (not including) 1.0.
    pub fn unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
}

impl Default for Random {
    fn default() -> Self {
        Random::new()
    }
}

/// Sends particles out in every direction from (x, y), each in a bright colour.
pub fn spawn_firework(particles: &mut Vec<Particle>, x: f32, y: f32, random: &mut Random) {
    let room = MAX_PARTICLES.saturating_sub(particles.len());
    for _ in 0..PARTICLES_PER_FIREWORK.min(room) {
        let angle = random.unit() * std::f32::consts::TAU;
        let speed = 1.0 + random.unit() * 4.0;
        let colour = 0x0080_8080 | (random.next_u32() & 0x00FF_FFFF); // never too dark
        particles.push(Particle { x, y, vx: speed * angle.cos(), vy: speed * angle.sin(), colour });
    }
}

/// Moves every particle, then forgets the ones that left the window on any side.
pub fn update_particles(particles: &mut Vec<Particle>) {
    for p in particles.iter_mut() {
        p.vy += GRAVITY;
        p.x += p.vx;
        p.y += p.vy;
    }
    particles.retain(|p| p.x >= 0.0 && p.x < WIDTH as f32 && p.y < HEIGHT as f32);
}

// ---- Drawing: writing numbers into the pixel buffer ---------------------------------------------

/// Sets one pixel, if it's inside the picture. Everything else is built on this.
fn set_pixel(buffer: &mut [u32], x: i32, y: i32, colour: u32) {
    if (0..WIDTH as i32).contains(&x) && (0..HEIGHT as i32).contains(&y) {
        buffer[y as usize * WIDTH + x as usize] = colour;
    }
}

/// A filled circle: every pixel whose distance from the centre is at most the
/// radius. Comparing squares (dx² + dy² ≤ r²) avoids a square root per pixel.
pub fn draw_circle(buffer: &mut [u32], cx: f32, cy: f32, radius: f32, colour: u32) {
    let (cx, cy, r) = (cx.round() as i32, cy.round() as i32, radius.round() as i32);
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy <= r * r {
                set_pixel(buffer, x, y, colour);
            }
        }
    }
}

pub fn draw_rectangle(buffer: &mut [u32], x: f32, y: f32, width: f32, height: f32, colour: u32) {
    let (x, y) = (x.round() as i32, y.round() as i32);
    for py in y..y + height.round() as i32 {
        for px in x..x + width.round() as i32 {
            set_pixel(buffer, px, py, colour);
        }
    }
}

/// Draws one complete frame: background, particles, then the face on top.
pub fn draw(buffer: &mut [u32], face: &Face, particles: &[Particle]) {
    buffer.fill(BACKGROUND);
    for p in particles {
        set_pixel(buffer, p.x as i32, p.y as i32, p.colour);
    }
    draw_circle(buffer, face.x, face.y, FACE_RADIUS, FACE_COLOUR);
    for side in [-1.0, 1.0] {
        draw_circle(buffer, face.x + side * EYE_OFFSET.0, face.y + EYE_OFFSET.1, EYE_RADIUS, EYE_COLOUR);
    }
    let (width, height) = MOUTH_SIZE;
    draw_rectangle(buffer, face.x - width / 2.0, face.y + MOUTH_OFFSET_Y, width, height, MOUTH_COLOUR);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(buffer: &[u32], x: f32, y: f32) -> u32 {
        buffer[y as usize * WIDTH + x as usize]
    }

    #[test]
    fn colours_are_drawn_where_they_belong() {
        let mut buffer = vec![0; WIDTH * HEIGHT];
        let face = Face::new();
        draw(&mut buffer, &face, &[]);
        assert_eq!(pixel(&buffer, 5.0, 5.0), BACKGROUND);
        assert_eq!(pixel(&buffer, face.x, face.y), FACE_COLOUR);
        assert_eq!(pixel(&buffer, face.x - 30.0, face.y - 20.0), EYE_COLOUR);
        assert_eq!(pixel(&buffer, face.x, face.y + 45.0), MOUTH_COLOUR);
    }

    #[test]
    fn a_circle_has_the_right_shape() {
        let mut buffer = vec![0; WIDTH * HEIGHT];
        draw_circle(&mut buffer, 100.0, 100.0, 10.0, 1);
        assert_eq!(pixel(&buffer, 110.0, 100.0), 1); // on the edge
        assert_eq!(pixel(&buffer, 111.0, 100.0), 0); // just outside
        assert_eq!(pixel(&buffer, 108.0, 108.0), 0); // the corner of the square around it
    }

    #[test]
    fn drawing_off_screen_never_panics() {
        let mut buffer = vec![0; WIDTH * HEIGHT];
        draw_circle(&mut buffer, -50.0, -50.0, 80.0, 1);
        draw_circle(&mut buffer, 10_000.0, 10_000.0, 80.0, 1);
        draw_rectangle(&mut buffer, WIDTH as f32 - 5.0, HEIGHT as f32 - 5.0, 100.0, 100.0, 1);
    }

    #[test]
    fn the_face_never_leaves_the_window() {
        // The mouse in every corner, for a long time: the case that used to
        // crash (an unsigned subtraction below zero) and push the face outside.
        let corners = [(0.0, 0.0), (WIDTH as f32, 0.0), (0.0, HEIGHT as f32), (WIDTH as f32, HEIGHT as f32)];
        for corner in corners {
            let mut face = Face::new();
            for _ in 0..500 {
                face.update(Some(corner));
                assert!((FACE_RADIUS..=WIDTH as f32 - FACE_RADIUS).contains(&face.x), "{face:?}");
                assert!((FACE_RADIUS..=HEIGHT as f32 - FACE_RADIUS).contains(&face.y), "{face:?}");
            }
        }
    }

    #[test]
    fn without_a_mouse_it_bounces() {
        let mut face = Face { x: WIDTH as f32 - FACE_RADIUS - 1.0, y: 240.0, vx: 5.0, vy: 0.0 };
        face.update(None);
        assert_eq!(face.vx, -5.0); // reversed at the right wall
        assert_eq!(face.x, WIDTH as f32 - FACE_RADIUS); // and kept inside
    }

    #[test]
    fn particles_are_limited_and_cleaned_up() {
        let mut random = Random(42);
        let mut particles = Vec::new();
        for _ in 0..100 {
            spawn_firework(&mut particles, 320.0, 240.0, &mut random);
        }
        assert_eq!(particles.len(), MAX_PARTICLES); // never more than the limit
        for _ in 0..1_000 {
            update_particles(&mut particles);
        }
        assert!(particles.is_empty()); // every particle eventually falls out
    }
}
