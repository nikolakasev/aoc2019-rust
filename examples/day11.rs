use aoc2019_rust::intcode::async_computer;
use cgmath::Vector2;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::mpsc::{self, Receiver, Sender},
};
use std::{
    fmt::{self, Formatter},
    thread,
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tile {
    White = 1,
    Black = 0,
}

impl Tile {
    fn from(v: i64) -> Tile {
        if v == 0 {
            Tile::Black
        } else if v == 1 {
            Tile::White
        } else {
            panic!("can't handle {}", v)
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            Tile::Black => ".",
            Tile::White => "#",
        };

        write!(f, "{}", string)
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            Direction::Right => "►",
            Direction::Left => "◄",
            Direction::Up => "▲",
            Direction::Down => "▼",
        };

        write!(f, "{}", string)
    }
}

impl Direction {
    fn turn(&self, turn: Turn) -> Direction {
        match self {
            Direction::Up => {
                if turn == Turn::Left {
                    Direction::Left
                } else {
                    Direction::Right
                }
            }
            Direction::Down => {
                if turn == Turn::Left {
                    Direction::Right
                } else {
                    Direction::Left
                }
            }
            Direction::Left => {
                if turn == Turn::Left {
                    Direction::Down
                } else {
                    Direction::Up
                }
            }
            Direction::Right => {
                if turn == Turn::Left {
                    Direction::Up
                } else {
                    Direction::Down
                }
            }
        }
    }

    fn one_step_from(&self, location: Vector2<i8>) -> Vector2<i8> {
        match self {
            Direction::Up => Vector2::new(location.x, location.y - 1),
            Direction::Down => Vector2::new(location.x, location.y + 1),
            Direction::Left => Vector2::new(location.x - 1, location.y),
            Direction::Right => Vector2::new(location.x + 1, location.y),
        }
    }
}

#[derive(PartialEq)]
enum Turn {
    Left,
    Right,
}

impl Turn {
    fn from(v: i64) -> Turn {
        if v == 0 {
            Turn::Left
        } else if v == 1 {
            Turn::Right
        } else {
            panic!("can't handle {}", v)
        }
    }
}

enum Sprite {
    East,
    West,
    North,
    South,
    White,
    Black,
}

impl fmt::Display for Sprite {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let string = match self {
            Sprite::East => "►",
            Sprite::West => "◄",
            Sprite::North => "▲",
            Sprite::South => "▼",
            Sprite::White => "#",
            Sprite::Black => ".",
        };

        write!(f, "{}", string)
    }
}

struct World {
    known: HashMap<Vector2<i8>, Tile>,
    painted: HashSet<Vector2<i8>>,
}

impl World {
    fn new() -> World {
        World {
            //initialize the hash map from an array
            known: [(Vector2::new(0, 0), Tile::Black)]
                .iter()
                .cloned()
                .collect(),
            painted: HashSet::new(),
        }
    }

    fn known_bounds(&self) -> (Vector2<i8>, Vector2<i8>) {
        //todo how efficint is this? is there a way to reuse the iterator?
        let min_x = self
            .known
            .clone()
            .into_iter()
            .map(|(location, _)| location)
            .map(|v| v.x)
            .min()
            .unwrap();
        let max_x = self
            .known
            .clone()
            .into_iter()
            .map(|(location, _)| location)
            .map(|v| v.x)
            .max()
            .unwrap();
        let min_y = self
            .known
            .clone()
            .into_iter()
            .map(|(location, _)| location)
            .map(|v| v.y)
            .min()
            .unwrap();
        let max_y = self
            .known
            .clone()
            .into_iter()
            .map(|(location, _)| location)
            .map(|v| v.y)
            .max()
            .unwrap();

        (Vector2::new(min_x, min_y), Vector2::new(max_x, max_y))
    }
}

fn main() {
    let input_day11 = "3,8,1005,8,345,1106,0,11,0,0,0,104,1,104,0,3,8,102,-1,8,10,1001,10,1,10,4,10,108,1,8,10,4,10,102,1,8,28,1006,0,94,2,106,5,10,1,1109,12,10,3,8,1002,8,-1,10,1001,10,1,10,4,10,1008,8,1,10,4,10,101,0,8,62,1,103,6,10,1,108,12,10,3,8,102,-1,8,10,1001,10,1,10,4,10,1008,8,0,10,4,10,102,1,8,92,2,104,18,10,2,1109,2,10,2,1007,5,10,1,7,4,10,3,8,102,-1,8,10,1001,10,1,10,4,10,108,0,8,10,4,10,102,1,8,129,2,1004,15,10,2,1103,15,10,2,1009,6,10,3,8,102,-1,8,10,1001,10,1,10,4,10,1008,8,1,10,4,10,101,0,8,164,2,1109,14,10,1,1107,18,10,1,1109,13,10,1,1107,11,10,3,8,102,-1,8,10,101,1,10,10,4,10,108,0,8,10,4,10,1001,8,0,201,2,104,20,10,1,107,8,10,1,1007,5,10,3,8,102,-1,8,10,101,1,10,10,4,10,1008,8,1,10,4,10,101,0,8,236,3,8,1002,8,-1,10,1001,10,1,10,4,10,108,0,8,10,4,10,1001,8,0,257,3,8,102,-1,8,10,101,1,10,10,4,10,108,1,8,10,4,10,102,1,8,279,1,107,0,10,1,107,16,10,1006,0,24,1,101,3,10,3,8,102,-1,8,10,101,1,10,10,4,10,108,0,8,10,4,10,1002,8,1,316,2,1108,15,10,2,4,11,10,101,1,9,9,1007,9,934,10,1005,10,15,99,109,667,104,0,104,1,21101,0,936995730328,1,21102,362,1,0,1105,1,466,21102,1,838210728716,1,21101,373,0,0,1105,1,466,3,10,104,0,104,1,3,10,104,0,104,0,3,10,104,0,104,1,3,10,104,0,104,1,3,10,104,0,104,0,3,10,104,0,104,1,21102,1,235350789351,1,21101,0,420,0,1105,1,466,21102,29195603035,1,1,21102,1,431,0,1105,1,466,3,10,104,0,104,0,3,10,104,0,104,0,21101,0,825016079204,1,21101,0,454,0,1105,1,466,21101,837896786700,0,1,21102,1,465,0,1106,0,466,99,109,2,21201,-1,0,1,21101,0,40,2,21102,1,497,3,21101,0,487,0,1105,1,530,109,-2,2106,0,0,0,1,0,0,1,109,2,3,10,204,-1,1001,492,493,508,4,0,1001,492,1,492,108,4,492,10,1006,10,524,1101,0,0,492,109,-2,2105,1,0,0,109,4,2102,1,-1,529,1207,-3,0,10,1006,10,547,21102,1,0,-3,21201,-3,0,1,22102,1,-2,2,21101,1,0,3,21102,1,566,0,1105,1,571,109,-4,2106,0,0,109,5,1207,-3,1,10,1006,10,594,2207,-4,-2,10,1006,10,594,21201,-4,0,-4,1106,0,662,21201,-4,0,1,21201,-3,-1,2,21202,-2,2,3,21101,613,0,0,1105,1,571,22101,0,1,-4,21101,0,1,-1,2207,-4,-2,10,1006,10,632,21101,0,0,-1,22202,-2,-1,-2,2107,0,-3,10,1006,10,654,22101,0,-1,1,21102,654,1,0,105,1,529,21202,-2,-1,-2,22201,-4,-2,-4,109,-5,2105,1,0";

    let (tx_input, rx_input): (Sender<i64>, Receiver<i64>) = mpsc::channel();
    let (tx_output, rx_output): (Sender<i64>, Receiver<i64>) = mpsc::channel();

    let _a = thread::spawn(move || {
        async_computer(input_day11, "hull-painting robot", rx_input, tx_output)
    });

    let mut location = Vector2::new(0, 0);
    let mut direction = Direction::Up;

    let mut element = 1;
    let mut tile = Tile::Black;

    let mut world = World::new();

    tx_input.send(Tile::White as i64);

    loop {
        match rx_output.recv() {
            Ok(v) => {
                //paint
                if element == 1 {
                    tile = Tile::from(v);
                    world.known.insert(location, tile);
                    world.painted.insert(location);

                    element += 1;
                }
                //turn
                else {
                    let turn = Turn::from(v);

                    direction = direction.turn(turn);
                    // println!("direction is {} because of {}", direction, v);

                    let new_location = move_robot(location, &direction);
                    println!("new location is {:?}", new_location);

                    println!("c {}", world.painted.len());

                    match world.known.get(&new_location) {
                        None => {
                            println!("{:?} is new", new_location);

                            tx_input.send(Tile::Black as i64);
                        }
                        Some(t) => {
                            println!("{:?} is known with tile {}", new_location, t);

                            tile = *t;
                            tx_input.send(tile as i64);
                        }
                    }

                    location = new_location;

                    element = 1;
                }
                continue;
            }
            Err(_) => break,
        }
    }

    print_world(&world)
}

fn move_robot(robot_location: Vector2<i8>, direction: &Direction) -> Vector2<i8> {
    direction.one_step_from(robot_location)
}

fn print_world(world: &World) {
    let (min, max) = world.known_bounds();

    // The escape sequence `\x1B[2J` clear the screen
    let lines: String = std::iter::once("\x1B[2J\n")
        .chain((min.y..=max.y).flat_map(|y| {
            (min.x..=max.x)
                .map(move |x| match world.known.get(&Vector2::new(x, y)) {
                    //todo use the Display trait that Tile implements
                    Some(tile) => match tile {
                        Tile::White => "#",
                        Tile::Black => ".",
                    },
                    None => ".",
                })
                .chain(std::iter::once("\n"))
        }))
        .collect();

    print!("{}", lines);
}
