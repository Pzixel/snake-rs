use std::{io::stdout, sync::{Arc, Mutex}, thread};

use rand::RngCore;
use crossterm::{cursor, event::{read, Event, KeyCode, KeyEvent, KeyModifiers}, execute, terminal::enable_raw_mode};

const N: usize = 16;
const M: usize = 16;

// snake game

struct Game {
    snake: Vec<(usize, usize)>,
    food: (usize, usize),
    direction: Direction,
    generator: rand::rngs::ThreadRng,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Game {
    fn new() -> Self {
        let mut result = Self {
            snake: vec![(N / 2, M / 2)],
            food: Default::default(),
            direction: Direction::Right,
            generator: rand::thread_rng(),
        };
        result.food = result.next_food_position();
        result
    }

    fn update(&mut self) {
        let head = self.snake[0];
        let (dx, dy) = match self.direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        let new_head = (
            (head.0 as isize + dx + N as isize) as usize % N,
            (head.1 as isize + dy + M as isize) as usize % M,
        );

        self.snake.insert(0, new_head);
        if self.snake[0] == self.food {
            self.food = self.next_food_position();
        } else {
            self.snake.pop();
        }
    }

    fn draw(&self) {
        for y in 0..M {
            for x in 0..N {
                if self.snake.contains(&(x, y)) {
                    print!("X");
                } else if (x, y) == self.food {
                    print!("O");
                } else {
                    print!(" ");
                }
            }
            execute!(stdout(), cursor::MoveToNextLine(1)).unwrap();
        }
        execute!(stdout(), cursor::MoveTo(0, 0)).unwrap();
    }

    fn next_food_position(&mut self) -> (usize, usize) {
        let mut x = self.generator.next_u32() as usize % N;
        let mut y = self.generator.next_u32() as usize % M;
        while self.snake.contains(&(x, y)) {
            x = self.generator.next_u32() as usize % N;
            y = self.generator.next_u32() as usize % M;
        }
        (x, y)
    }
}

fn main() {
    enable_raw_mode().unwrap();

    let mut game = Game::new();
    let current_direction = Arc::new(Mutex::new(game.direction));
    

    thread::spawn({
        let current_direction = current_direction.clone();
        move || {            
            loop {
                match read().unwrap() {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        std::process::exit(0);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W'),
                        ..
                    }) => {
                        *current_direction.lock().unwrap() = Direction::Up;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S'),
                        ..
                    }) => {
                        *current_direction.lock().unwrap() = Direction::Down;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A'),
                        ..
                    }) => {
                        *current_direction.lock().unwrap() = Direction::Left;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D'),
                        ..
                    }) => {
                        *current_direction.lock().unwrap() = Direction::Right;
                    }
                _ => {}
            }
        }}
    });

    loop {
        game.draw();
        game.update();
        std::thread::sleep(std::time::Duration::from_millis(100));
        game.direction = *current_direction.lock().unwrap();
    }
}
