// Snake for phones (and computers), made with the macroquad game library.
//
// The game rules (Direction, Point, Game) are the same as in the terminal version,
// ../src/main.rs. What's new is the drawing (coloured squares instead of characters)
// and the controls (swipes instead of arrow keys).
//
// Controls: swipe to steer (drag with the mouse on a computer), tap to play again.
//           Arrow keys and WASD work too, if there's a keyboard.

use std::collections::VecDeque;

// The prelude brings in what most games need: drawing, colours, input, time, ...
// It also brings in macroquad's own `rand` module, used in random_free_point.
use macroquad::prelude::*;

// Size of the playing field, not counting the walls.
const WIDTH: i32 = 30;
const HEIGHT: i32 = 15;

// How long one step of the snake takes, in seconds. Smaller = faster game.
// (macroquad measures time in seconds, as an f32: a 32-bit float, Java's `float`.)
const TICK: f32 = 0.15;

// ---------------------------------------------------------------------------
// The game rules: the same as in the terminal version.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

// A position on the field. (0, 0) is the top-left corner.
#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

struct Game {
    snake: VecDeque<Point>,    // the snake's body, head first
    direction: Direction,      // the direction the snake moved in last step
    next_direction: Direction, // the direction the player asked for
    food: Point,
    score: u32,
    game_over: bool,
}

impl Game {
    fn new() -> Game {
        let mut snake = VecDeque::new();
        snake.push_back(Point { x: 5, y: 7 }); // head
        snake.push_back(Point { x: 4, y: 7 });
        snake.push_back(Point { x: 3, y: 7 }); // tail

        let food = Game::random_free_point(&snake);

        Game {
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            score: 0,
            game_over: false,
        }
    }

    // Called when the player swipes or presses an arrow key.
    fn turn(&mut self, wanted: Direction) {
        // The snake may not reverse into itself.
        if wanted != self.direction.opposite() {
            self.next_direction = wanted;
        }
    }

    // Moves the snake one step. Called once per tick.
    fn update(&mut self) {
        if self.game_over {
            return;
        }

        self.direction = self.next_direction;

        let head = self.snake[0];
        let new_head = match self.direction {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };

        let eating = new_head == self.food;

        // Moving = add a new head and drop the tail. When eating, keep the tail.
        if !eating {
            self.snake.pop_back();
        }

        let hit_wall = new_head.x < 0 || new_head.x >= WIDTH || new_head.y < 0 || new_head.y >= HEIGHT;
        let hit_itself = self.snake.contains(&new_head);
        if hit_wall || hit_itself {
            self.game_over = true;
            return;
        }

        self.snake.push_front(new_head);

        if eating {
            self.score += 1;
            self.food = Game::random_free_point(&self.snake);
        }
    }

    // Picks a random square that the snake isn't on.
    fn random_free_point(snake: &VecDeque<Point>) -> Point {
        loop {
            // macroquad's gen_range(low, high) gives a number from low up to high-1.
            let p = Point {
                x: rand::gen_range(0, WIDTH),
                y: rand::gen_range(0, HEIGHT),
            };
            if !snake.contains(&p) {
                return p;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

// Turns finger movements into directions.
struct Swipe {
    // Where the current swipe started. `Option` because there may be no finger
    // on the screen: then it's `None` (Rust's way of saying null).
    start: Option<Vec2>,
}

impl Swipe {
    // Called every frame. Returns a direction once the finger has moved far enough.
    fn direction(&mut self) -> Option<Direction> {
        // macroquad reports a finger on the screen as the left mouse button,
        // so the same code works with a mouse on a computer.
        if is_mouse_button_pressed(MouseButton::Left) {
            self.start = Some(Vec2::from(mouse_position()));
        }
        if !is_mouse_button_down(MouseButton::Left) {
            self.start = None;
            return None;
        }

        // `?` on an Option: if start is None, return None from this function right away.
        let start = self.start?;
        let now = Vec2::from(mouse_position());
        let moved = now - start;

        // Small finger movements don't count: at least 5% of the screen's short side.
        let min_distance = screen_width().min(screen_height()) * 0.05;
        if moved.length() < min_distance {
            return None;
        }

        // Start the next swipe from here, so the player can steer again
        // without lifting the finger.
        self.start = Some(now);

        // Whichever way the finger moved more decides: sideways or up/down.
        // (Screen y grows downwards, like in Java's Swing.)
        if moved.x.abs() > moved.y.abs() {
            Some(if moved.x > 0.0 { Direction::Right } else { Direction::Left })
        } else {
            Some(if moved.y > 0.0 { Direction::Down } else { Direction::Up })
        }
    }
}

// The direction of an arrow key or WASD key pressed in this frame, if any.
fn key_direction() -> Option<Direction> {
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        Some(Direction::Up)
    } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        Some(Direction::Down)
    } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        Some(Direction::Left)
    } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        Some(Direction::Right)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

fn draw(game: &Game) {
    clear_background(BLACK);

    // Make the squares as big as possible, while the field fits on the screen
    // with room for the walls and the score line.
    let cell = (screen_width() / (WIDTH + 2) as f32).min(screen_height() / (HEIGHT + 4) as f32);

    // Top-left corner of the field, chosen so the field sits in the middle.
    let left = (screen_width() - WIDTH as f32 * cell) / 2.0;
    let top = (screen_height() - HEIGHT as f32 * cell) / 2.0 + cell;

    // The field and the wall around it.
    let field_width = WIDTH as f32 * cell;
    let field_height = HEIGHT as f32 * cell;
    draw_rectangle(left, top, field_width, field_height, Color::from_rgba(20, 30, 25, 255));
    let wall = cell * 0.3;
    draw_rectangle_lines(
        left - wall,
        top - wall,
        field_width + 2.0 * wall,
        field_height + 2.0 * wall,
        wall * 2.0, // macroquad draws half of the line's thickness inside, half outside
        GRAY,
    );

    // The food: a red dot in the middle of its square.
    let food_x = left + (game.food.x as f32 + 0.5) * cell;
    let food_y = top + (game.food.y as f32 + 0.5) * cell;
    draw_circle(food_x, food_y, cell * 0.4, RED);

    // The snake: one square per segment, with a small gap so the segments are visible.
    // `.iter().enumerate()` gives each element with its index, like a Java for (int i...) loop.
    for (i, part) in game.snake.iter().enumerate() {
        let color = if i == 0 { LIME } else { DARKGREEN }; // the head is brighter
        let gap = cell * 0.08;
        draw_rectangle(
            left + part.x as f32 * cell + gap,
            top + part.y as f32 * cell + gap,
            cell - 2.0 * gap,
            cell - 2.0 * gap,
            color,
        );
    }

    // The score, above the field. draw_text's y is where the bottom of the letters goes.
    let text_size = cell * 1.2;
    draw_text(&format!("Score: {}", game.score), left, top - cell * 0.6, text_size, WHITE);

    if game.game_over {
        // Darken everything, then write the message in the middle.
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.6));
        draw_centered_text("GAME OVER", screen_height() / 2.0 - cell, text_size * 2.0, WHITE);
        draw_centered_text("Tap to play again", screen_height() / 2.0 + cell * 1.5, text_size, LIGHTGRAY);
    }
}

fn draw_centered_text(text: &str, y: f32, size: f32, color: Color) {
    let width = measure_text(text, None, size as u16, 1.0).width;
    draw_text(text, (screen_width() - width) / 2.0, y, size, color);
}

// ---------------------------------------------------------------------------
// Starting the game
// ---------------------------------------------------------------------------

// The C library's _exit function: ends the process immediately. This declares it so
// Rust can call it, like a `native` method in Java. `-> !` means "never returns".
//
// Why not Rust's std::process::exit? That runs cleanup code first, and on Android the
// cleanup crashes, because Android's own drawing threads are still running at that point.
extern "C" {
    fn _exit(status: i32) -> !;
}

// The window settings on a computer. On a phone the game always fills the screen.
fn window_conf() -> Conf {
    Conf {
        window_title: "Snake".to_owned(),
        window_width: 960,
        window_height: 540,
        high_dpi: true,
        // `..Default::default()` = every other setting keeps its default value
        ..Default::default()
    }
}

// `#[macroquad::main]` is an attribute macro: it writes the real `main` function,
// which sets up the window (or the phone screen) and then runs the code below.
//
// `async` and `.await` let macroquad pause our loop between frames:
// `next_frame().await` means "show what I drew, and continue when it's time for
// the next frame". You don't need to know more about async to use it here.
#[macroquad::main(window_conf)]
async fn main() {
    // macroquad's random numbers start from the same seed every time; seed them
    // with the current time, so the food appears in different places every game.
    rand::srand(macroquad::miniquad::date::now() as u64);

    let mut game = Game::new();
    let mut swipe = Swipe { start: None };
    let mut time_since_step = 0.0;

    // The game loop, which runs once per frame (usually 60 times per second):
    // read the controls, move the snake when a tick has passed, draw.
    loop {
        // Android's back button closes the game, like in most Android apps.
        // We end the whole process, not just the screen: macroquad can only start once
        // per process, so the next start must get a fresh one. (Never true on a computer.)
        if is_key_pressed(KeyCode::Back) {
            // `unsafe`: the compiler can't check functions written in C,
            // so we have to promise that calling it is fine. (It is: it only takes a number.)
            unsafe { _exit(0) };
        }

        // Called every frame, also after game over, so it always knows where the finger is.
        let swiped = swipe.direction();

        if game.game_over {
            if is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::R) {
                game = Game::new();
                time_since_step = 0.0;
            }
        } else if let Some(direction) = swiped.or(key_direction()) {
            // `.or(...)`: use the swipe if there was one, otherwise the key.
            game.turn(direction);
        }

        // get_frame_time() is the number of seconds since the previous frame.
        time_since_step += get_frame_time();
        if time_since_step >= TICK {
            time_since_step = 0.0;
            game.update();
        }

        draw(&game);
        next_frame().await;
    }
}
