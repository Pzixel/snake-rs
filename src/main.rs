use std::{io::stdout, sync::{Arc, Mutex}, thread};

use rand::RngCore;
use crossterm::{cursor, event::{read, Event, KeyCode, KeyEvent, KeyModifiers}, execute, terminal::{disable_raw_mode, enable_raw_mode}};

const N: usize = 16;
const M: usize = 16;

// snake game

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tile {
    Empty,
    Snake(Direction),
    Food,
}

struct Game {
    map: [[Tile; N]; M],
    head: (usize, usize),
    tail: (usize, usize),
    direction: Direction,
    generator: rand::rngs::ThreadRng,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct ValidDirection(Direction);

impl ValidDirection {
    fn new(old_direction: Direction, direction: Direction) -> Self {
        match (old_direction, direction) {
            (Direction::Up, Direction::Down) => Self(old_direction),
            (Direction::Down, Direction::Up) => Self(old_direction),
            (Direction::Left, Direction::Right) => Self(old_direction),
            (Direction::Right, Direction::Left) => Self(old_direction),
            _ => Self(direction),
        }
    }
}

impl Game {
    fn new() -> Self {
        let mut map = [[Tile::Empty; N]; M];
        let head = (N / 2, M / 2);
        let tail = head;
        let direction = Direction::Right;
        map[head.1][head.0] = Tile::Snake(direction);

        let mut result = Self {
            head,
            tail,
            direction,
            generator: rand::thread_rng(),
            map,
        };
        result.update_food();
        result
    }

    fn update(&mut self) -> Result<(), String> {
        let Tile::Snake(old_direction) = self.map[self.head.1][self.head.0] else { unreachable!() };
        let valid_direction = ValidDirection::new(old_direction, self.direction);
        self.direction = valid_direction.0;
        let old_head = self.head;
        let old_tail = self.tail;
        let new_head = Self::get_direction_index(self.head, valid_direction);
        match self.map[new_head.1][new_head.0] {
            Tile::Empty => {
                self.map[new_head.1][new_head.0] = Tile::Snake(self.direction);
                self.map[old_head.1][old_head.0] = Tile::Snake(self.direction);
                self.head = new_head;

                if old_head == old_tail {
                    self.tail = new_head;
                } else {
                    let Tile::Snake(tail_tile) = self.map[self.tail.1][self.tail.0] else {
                        return Err(format!("Tail is not a snake: {:?}. Head: {:?}. Tail: {:?}", self.map[self.tail.1][self.tail.0], self.head, self.tail));
                    };

                    let new_tail = Self::get_direction_index(self.tail, ValidDirection::new(tail_tile, tail_tile));
                        self.map[self.tail.1][self.tail.0] = Tile::Empty;

                        self.tail = new_tail;
                }


                self.map[old_tail.1][old_tail.0] = Tile::Empty;
            }
            Tile::Food => {
                self.map[new_head.1][new_head.0] = Tile::Snake(self.direction);
                self.map[old_head.1][old_head.0] = Tile::Snake(self.direction);
                self.head = new_head;
                self.update_food();
            }
            Tile::Snake(_) => {
                return Err("Game over".into());
            }
        }
        disable_raw_mode().unwrap();
        println!("{:?}: {:?} {:?}: {:?}", 
            self.head, 
            self.map[self.head.1][self.head.0],
            self.tail,
            self.map[self.tail.1][self.tail.0]
        );
        enable_raw_mode().unwrap();
        Ok(())
    }

    fn get_direction_index(position: (usize, usize), ValidDirection(direction): ValidDirection) -> (usize, usize) {
        let (dx, dy) = match direction {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        Self::get_bounded_index(position, dx, dy)
    }

    fn get_bounded_index(position: (usize, usize), dx: isize, dy: isize) -> (usize, usize) {
        (
            (position.0 as isize + dx + N as isize) as usize % N,
            (position.1 as isize + dy + M as isize) as usize % M,
        )
    }

    fn draw(&self) {
        for y in 0..M {
            for x in 0..N {
                match self.map[y][x] {
                    Tile::Empty => print!(" "),
                    Tile::Snake(x) => match x {
                        Direction::Up => print!("^"),
                        Direction::Down => print!("v"),
                        Direction::Left => print!("<"),
                        Direction::Right => print!(">"),
                    },
                    Tile::Food => print!("*"),
                }
            }
            execute!(stdout(), cursor::MoveToNextLine(1)).unwrap();
        }
        execute!(stdout(), cursor::MoveTo(0, 0)).unwrap();
    }

    fn update_food(&mut self) {
        let (x, y) = self.next_food_position();
        self.map[y][x] = Tile::Food;
    }

    fn next_food_position(&mut self) -> (usize, usize) {
        let mut x = self.generator.next_u32() as usize % N;
        let mut y = self.generator.next_u32() as usize % M;
        while !matches!(self.map[y][x], Tile::Empty) {
            x = self.generator.next_u32() as usize % N;
            y = self.generator.next_u32() as usize % M;
        }
        (x, y)
    }
}

fn play() -> Result<(), String> {
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
        game.direction = *current_direction.lock().unwrap();
        game.draw();
        game.update()?;
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn main() {
    enable_raw_mode().unwrap();
    if let Err(e) = play() {
        disable_raw_mode().unwrap();
        eprintln!("{}\n\n", e);
    }
}
