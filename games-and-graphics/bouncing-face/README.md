<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Bouncing face

A cartoon face in a window that follows your mouse, with fireworks bursting around it. When the mouse leaves the window, the face bounces around on its own. Press **Esc** to quit.

It's a small program, but it shows how every computer game and animation works underneath: a list of numbers becomes a picture, many times per second.

## Run it

```bash
cargo run --release
cargo test
```

It needs a desktop to open a window in. On a machine without one, such as a server, it says so instead of crashing.

## How graphics work: a picture is a list of numbers

The window is 640 × 480 pixels, and the whole picture is one `Vec<u32>` with a number for each pixel:

```rust
let mut buffer = vec![0u32; WIDTH * HEIGHT];      // 307,200 pixels
buffer[y * WIDTH + x] = colour;                    // the pixel in column x, row y
```

The rows are stored one after another, so the pixel at column `x` in row `y` is at index `y * WIDTH + x`. This is called a **framebuffer**, and screens, image files and GPUs all work with this idea.

Each number is a colour, written as `0x00RRGGBB`: the amount of red, green and blue, from `00` to `FF` (0 to 255):

| Colour | Value | Red | Green | Blue |
|---|---|---|---|---|
| the face (warm yellow) | `0x00FFE066` | FF (full) | E0 | 66 |
| the mouth (red) | `0x00D03030` | D0 | 30 | 30 |
| the background (dark blue) | `0x00101828` | 10 | 18 | 28 |

## Drawing shapes

Every shape is drawn by deciding, pixel by pixel, which pixels belong to it. A circle is every pixel whose distance from the centre is at most the radius:

```rust
for y in cy - r..=cy + r {
    for x in cx - r..=cx + r {
        let (dx, dy) = (x - cx, y - cy);
        if dx * dx + dy * dy <= r * r {            // Pythagoras: inside the circle
            set_pixel(buffer, x, y, colour);
        }
    }
}
```

Comparing **squared** distances (`dx² + dy² ≤ r²`) gives the same answer as comparing distances, without a square root for every pixel. `set_pixel` ignores anything outside the window, so a shape that's partly off-screen is simply cut off, never a crash.

The face is three circles (the head and two eyes) and a rectangle (the mouth), drawn in that order: later shapes cover earlier ones, like layers of paint.

## The game loop

Every game, from Pong to the newest 3D title, repeats the same four steps, many times per second:

```rust
while window.is_open() && !window.is_key_down(Key::Escape) {
    let mouse = window.get_mouse_pos(MouseMode::Discard);   // 1. read the input
    face.update(mouse);                                      // 2. update the world
    scene::update_particles(&mut particles);
    scene::draw(&mut buffer, &face, &particles);             // 3. draw it into the buffer
    window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;      // 4. show it, wait for the next frame
}
```

`window.set_target_fps(60)` makes step 4 wait, so the loop runs 60 times per second: about 16.7 milliseconds per frame. Each frame redraws **everything** from scratch, starting with the background. Motion is just slightly different pictures, shown quickly one after another.

## Movement

The face has a position and a velocity, in pixels and pixels per frame, stored as `f32` so it can move by fractions of a pixel and glide smoothly.

- **With the mouse in the window**, the velocity points at the mouse: one twelfth of the remaining distance per frame. It moves fast when it's far away, and slows down as it arrives.
- **Without the mouse**, it keeps its velocity and bounces: hitting a wall reverses that direction.
- **Either way**, after moving, the position is **clamped** so the whole face stays inside the window.

The fireworks are 300 particles each, flying out in random directions. Each frame, gravity adds a little downward speed. A particle that leaves the window is removed, and there's a hard limit of 5,000 particles, so the list can't grow forever however long the program runs.

## How the code is organised

| File | Contains |
|---|---|
| `scene.rs` | the face, the particles, the drawing functions and a tiny random number generator: everything **without** a window, so it's all testable |
| `main.rs` | opening the window and the game loop |

The tests check pixel colours in a drawn frame, the exact shape of a circle, drawing far off-screen, the face staying inside the window with the mouse in every corner, bouncing, and the particle limit.

## What was fixed

This program was first written in 2023, as `test-minifb`. While modernising it, these problems turned up:

| Problem | Fix |
|---|---|
| **Every colour was wrong.** The colours were written as `0xRRGGBBAA`, but the window expects `0x00RRGGBB`. "Light yellow" `0xFFFFCCFF` showed as lavender, the "black" eyes `0x000000FF` and the "red" mouth `0xFF0000FF` both showed as blue | colours in the right format, with a comment and a test that checks the drawn pixels |
| **It could crash.** With the mouse near the left edge, the face followed it until `x - 30` went below zero in an unsigned `usize`, which panics in a debug build | positions are `f32`, drawing is done in `i32`, and the face is clamped inside the window; a test keeps the mouse in each corner for 500 frames |
| the face could slide halfway out of the window: a bounce reversed the direction but still moved the face past the edge | clamped back inside on every bounce |
| with the mouse outside the window, the face just stopped | it bounces on its own |
| particles flying out sideways were kept until they fell off the bottom, and their number had no limit | removed as soon as they leave the window, at most 5,000 |
| a failure to open the window was a panic | a clear message, then exit |
| the frame timing was a hand-written `sleep` | `set_target_fps(60)` |
| the README named a folder that doesn't exist (`human-face-mouse-move`) and a missing licence file | rewritten |
| the `rand` crate, and `minifb` 0.24 | a tiny xorshift generator from the standard library's random seed, and `minifb` 0.29 |

Back to [Games and graphics](../)
