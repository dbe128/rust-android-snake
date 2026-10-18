// A simple terminal Snake game.
//
// Controls: arrow keys or WASD to steer, R to restart after game over, Q or Esc to quit.
//
// Quick Rust-for-Java notes:
//   - `struct` holds data (like the fields of a class); `impl` holds its methods.
//   - Variables are immutable by default; `let mut` makes them changeable.
//   - `&self` is a read-only `this`, `&mut self` is a `this` the method may modify.
//   - There are no exceptions: functions that can fail return `Result`, and the `?`
//     operator means "if this failed, return the error to my caller right now".
//   - The last expression in a function, without a `;`, is its return value.

use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::{cursor, execute, queue, style, terminal};
use rand::RngExt; // gives random number generators the random_range method

// Size of the playing field, not counting the walls.
const WIDTH: i32 = 30;
const HEIGHT: i32 = 15;

// How long one step of the snake takes. Smaller = faster game.
const TICK: Duration = Duration::from_millis(150);

// An enum, much like a Java enum.
// `derive` asks the compiler to generate code for us:
//   Clone, Copy -> the value is copied on assignment, like a Java `int`
//   PartialEq   -> allows comparing with `==` (Java's equals())
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Direction {
        // `match` is a switch that must cover every case, and it returns a value.
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
    // The snake's body, head first. VecDeque is a double-ended queue
    // (like Java's ArrayDeque): cheap to add at the front and remove at the back.
    snake: VecDeque<Point>,
    direction: Direction,      // the direction the snake moved in last step
    next_direction: Direction, // the direction the player asked for
    food: Point,
    score: u32,
    game_over: bool,
}

impl Game {
    // There are no constructors in Rust. By convention, a function called `new`
    // builds the value. (A function without `self` is like a Java static method.)
    fn new() -> Game {
        let mut snake = VecDeque::new();
        snake.push_back(Point { x: 5, y: 7 }); // head
        snake.push_back(Point { x: 4, y: 7 });
        snake.push_back(Point { x: 3, y: 7 }); // tail

        // Pick the food before building the Game: the line `snake,` below moves the
        // snake into the struct, after which the local variable can't be used anymore.
        let food = Game::random_free_point(&snake);

        // No `;` after the struct, so it is the return value.
        Game {
            snake,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food,
            score: 0,
            game_over: false,
        }
    }

    // Called when the player presses an arrow key.
    fn turn(&mut self, wanted: Direction) {
        // The snake may not reverse into itself. We compare against the direction it
        // actually moved last step, so two quick key presses can't sneak a U-turn in.
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

        // `self.snake[0]` is the head. Point is Copy, so `head` is a copy, not a reference.
        let head = self.snake[0];
        let new_head = match self.direction {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };

        let eating = new_head == self.food;

        // Moving = add a new head and drop the tail.
        // When eating, keep the tail: that's how the snake grows.
        // The tail is dropped before the collision check, so the head may move
        // into the square the tail is just leaving.
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
    // It takes the snake as a parameter instead of `&self`, so `new` can call it
    // before the Game exists. (No `self` = like a Java static method.)
    fn random_free_point(snake: &VecDeque<Point>) -> Point {
        let mut rng = rand::rng();
        loop {
            let p = Point {
                x: rng.random_range(0..WIDTH), // 0..WIDTH means 0 up to WIDTH-1
                y: rng.random_range(0..HEIGHT),
            };
            if !snake.contains(&p) {
                return p;
            }
        }
    }

    // What character to show at a given square of the field.
    fn char_at(&self, p: Point) -> char {
        if p == self.snake[0] {
            '@'
        } else if self.snake.contains(&p) {
            'o'
        } else if p == self.food {
            '*'
        } else {
            ' '
        }
    }

    // Draws the whole screen. `out: &mut impl Write` means "any writable thing",
    // similar to taking a java.io.Writer parameter.
    fn draw(&self, out: &mut impl Write) -> io::Result<()> {
        let wall = "#".repeat(WIDTH as usize + 2);

        // `queue!` only collects commands; `flush()` at the end sends them all at once,
        // which avoids flickering.
        queue!(out, cursor::MoveTo(0, 0), style::Print(&wall))?;
        for y in 0..HEIGHT {
            let mut row = String::from("#");
            for x in 0..WIDTH {
                row.push(self.char_at(Point { x, y }));
            }
            row.push('#');
            queue!(out, cursor::MoveTo(0, y as u16 + 1), style::Print(row))?;
        }
        queue!(out, cursor::MoveTo(0, HEIGHT as u16 + 1), style::Print(&wall))?;

        // `format!` works like Java's String.format, with {} as the placeholder.
        let status = if self.game_over {
            format!("GAME OVER! Score: {}   R = restart, Q = quit", self.score)
        } else {
            format!("Score: {}   Arrows/WASD = move, Q = quit", self.score)
        };
        queue!(
            out,
            cursor::MoveTo(0, HEIGHT as u16 + 3),
            terminal::Clear(terminal::ClearType::CurrentLine),
            style::Print(status)
        )?;

        out.flush()
    }
}

// The game loop: draw, wait one tick while listening for keys, move the snake, repeat.
fn run(out: &mut impl Write) -> io::Result<()> {
    let mut game = Game::new();

    loop {
        game.draw(out)?;

        let tick_start = Instant::now();
        while tick_start.elapsed() < TICK {
            // Wait for a key press, but no longer than what's left of this tick.
            let time_left = TICK.saturating_sub(tick_start.elapsed());
            if !event::poll(time_left)? {
                continue; // no key pressed in time
            }

            // `if let` runs the block only if the value matches the pattern:
            // here, only if the event is a key press (not a mouse event, resize, ...).
            if let Event::Key(key) = event::read()? {
                // Some terminals also report key releases; we only care about presses.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Char('w') => game.turn(Direction::Up),
                    KeyCode::Down | KeyCode::Char('s') => game.turn(Direction::Down),
                    KeyCode::Left | KeyCode::Char('a') => game.turn(Direction::Left),
                    KeyCode::Right | KeyCode::Char('d') => game.turn(Direction::Right),
                    KeyCode::Char('r') if game.game_over => game = Game::new(),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {} // `_` matches anything else: ignore other keys
                }
            }
        }

        game.update();
    }
}

fn main() -> io::Result<()> {
    let mut out = io::stdout();

    // Raw mode: get key presses immediately, without waiting for Enter and
    // without echoing them. The alternate screen is a separate, blank screen
    // that disappears when we leave it, so the terminal looks as before.
    terminal::enable_raw_mode()?;
    execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = run(&mut out);

    // Always restore the terminal, even if the game ended with an error.
    execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}
