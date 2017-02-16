extern crate fnv;
extern crate num;

#[macro_use]
extern crate generic_array;
extern crate typenum;

use num::{ToPrimitive, FromPrimitive};
use num::integer::Integer;

use generic_array::{GenericArray, ArrayLength};

use std::collections::HashMap;
use std::hash::{Hash, Hasher, BuildHasherDefault};
use fnv::FnvHasher;

use std::io::{self, Read};
use std::fmt;
use std::str::FromStr;
use std::path::Path;

const WALL_CHAR: char = '#';
const GOAL_CHAR: char = 'G';
const EMPTY_CHAR: char = ' ';

const MAX_ROBOTS: usize = 8;

type Robot = u8;

struct Robots<Pos, N: ArrayLength<Pos>>(GenericArray<Pos, N>);

impl<Pos, N> Clone for Robots<Pos, N>
    where Pos: Clone,
          N: ArrayLength<Pos>,
{
    fn clone(&self) -> Self {
        Robots(self.0.clone())
    }
}

impl<Pos, N> Copy for Robots<Pos, N>
    where Pos: Copy,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
}

impl<Pos, N> PartialEq for Robots<Pos, N>
    where Pos: Eq,
          N: ArrayLength<Pos>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().eq(other.0.iter())
    }
}

impl<Pos, N> Eq for Robots<Pos, N>
    where Pos: Eq,
          N: ArrayLength<Pos>,
{
}

impl<Pos, N> Hash for Robots<Pos, N>
    where Pos: Hash,
          N: ArrayLength<Pos>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub type Solution<Pos> = Vec<Move<Pos>>;

type MyHasher = BuildHasherDefault<FnvHasher>;

/// Two intended types for T:
/// - bool: The raw board as read from input, true index has a wall
/// - Endpoints: What indices can be reached from each index in each direction, disregarding where the robots are placed
pub struct Board<T, Pos, N>
    where N: ArrayLength<Pos>,
{
    board: Box<[T]>,
    robots: Robots<Pos, N>,
    goal: Pos,
    size: Pos,
}

/// Where will you end up by going in each direction
#[derive(Debug, Clone, Copy)]
pub struct Endpoints<Pos> {
    up: Pos,
    right: Pos,
    down: Pos,
    left: Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

/// Move a robot to an index and remember the direction for reconstructing the solution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move<Pos> {
    to: Pos,
    robot: Robot,
    dir: Direction,
}

/// Node used by the search consist of
/// - robots: Placement of the robots constituting the current state
/// - action: What Move brought us from the parent state to this one
struct Node<Pos, N>
    where N: ArrayLength<Pos>,
{
    robots: Robots<Pos, N>,
    action: Move<Pos>,
}

impl<Pos, N> Clone for Node<Pos, N>
    where Pos: Clone,
          N: ArrayLength<Pos>,
{
    fn clone(&self) -> Self {
        Node {
            robots: self.robots.clone(),
            action: self.action.clone(),
        }
    }
}

impl<Pos, N> Copy for Node<Pos, N>
    where Pos: Copy,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
}

/// Create a child node from the parent and the given move
fn child_node<Pos, N>(mut parent: Node<Pos, N>, m: Move<Pos>) -> Node<Pos, N>
    where Pos: Copy,
          N: ArrayLength<Pos>,
{
    parent.robots.0[m.robot as usize] = m.to;
    parent.action = m;
    parent
}

/// Iterative Deepening Depth-First Search
pub fn ids<Pos, N>(board: &Board<Endpoints<Pos>, Pos, N>) -> Solution<Pos>
    where Pos: Default + Hash + Copy + Integer + ToPrimitive,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
    let mut depth = 0;

    // Preallocate vectors for holding possible moves at each depth
    let mut moves_vecs: Vec<Vec<Move<Pos>>> = vec![
        Vec::with_capacity(4 * board.robots.0.len());
        depth as usize
    ];

    let mut seen = HashMap::default();

    loop {
        println!("DEPTH: {:2}", depth);

        if let Some(mut result) =
            depth_limited_search(board, depth, &mut moves_vecs, &mut seen)
        {
            result.reverse();
            return result;
        }

        depth += 1;
        moves_vecs.push(Vec::with_capacity(4 * board.robots.0.len()));
    }
}

/// Depth-first search to a given limit
fn depth_limited_search<Pos, N>
    (board: &Board<Endpoints<Pos>, Pos, N>,
     limit: u8,
     moves: &mut [Vec<Move<Pos>>],
     seen: &mut HashMap<Robots<Pos, N>, u8, MyHasher>)
     -> Option<Solution<Pos>>
    where Pos: Copy + Integer + Hash + ToPrimitive,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
    let initial = Node {
        robots: board.robots,
        // let the first move be by a non-existing robot
        action: Move::new(MAX_ROBOTS as u8, Pos::zero(), Direction::Up),
    };

    recursive_dls(board, initial, limit, moves, seen)
}

/// The actual search
fn recursive_dls<Pos, N>
    (board: &Board<Endpoints<Pos>, Pos, N>,
     node: Node<Pos, N>,
     limit: u8,
     moves_vecs: &mut [Vec<Move<Pos>>],
     seen: &mut HashMap<Robots<Pos, N>, u8, MyHasher>)
     -> Option<Solution<Pos>>
    where Pos: Integer + Copy + Hash + ToPrimitive,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
    if node.robots.0[0] == board.goal {
        println!("STATES SEEN: {}", seen.len());
        Some(vec![])
    } else if limit == 0 {
        None
    } else if *seen.get(&node.robots).unwrap_or(&0) >= limit {
        return None;
    } else {
        seen.insert(node.robots.clone(), limit);

        let previous_move = node.action;

        let (mut moves, rest) = moves_vecs.split_first_mut().unwrap();

        // Clear the moves from the previous node
        moves.clear();

        // Fill in the possible ones for this
        board.possible_moves(&node.robots, &mut moves);

        for action in moves {
            if action.robot != previous_move.robot ||
                !action.dir.is_opposite(&previous_move.dir)
            {
                let child = child_node(node, action.clone());

                let result = recursive_dls(board, child, limit-1, rest, seen);

                if let Some(mut res) = result {
                    res.push(child.action);
                    return Some(res);
                }
            }
        }

        None
    }
}

/// Parse a board from a string
impl<Pos, N> FromStr for Board<bool, Pos, N>
    where Pos: FromStr + fmt::Debug + fmt::Display + Integer + ToPrimitive + Copy + Default,
          Pos::Err: fmt::Debug,
          N: ArrayLength<Pos>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let size: Pos = lines.next().unwrap().parse().unwrap();
        lines.next().unwrap();
        let mut robots = GenericArray::new();
        let mut goal = Pos::zero();

        let mut board = Vec::with_capacity((size * size).to_usize().unwrap());
        let mut idx: Pos = Pos::zero();

        for line in lines {
            for c in line.chars() {
                match c {
                    EMPTY_CHAR => board.push(false),
                    WALL_CHAR => board.push(true),
                    d if d.is_digit(10) => {
                        let i = d.to_digit(10).unwrap();
                        robots[i as usize] = idx;
                        board.push(false);
                    },
                    GOAL_CHAR => {
                        goal = idx;
                        board.push(false);
                    },
                    c => return Err(format!("Unexpected character: {}", c)),
                }

                idx = idx + Pos::one();
            }
        }

        Ok(Board {
            board: board.into_boxed_slice(),
            robots: Robots(robots),
            size: size,
            goal: goal,
        })
    }
}

/// Display a board
impl<Pos, N> fmt::Display for Board<bool, Pos, N>
    where Pos: fmt::Display + ToPrimitive + Integer + Copy,
          N: ArrayLength<Pos>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(writeln!(f, "{}", self.size));
        try!(writeln!(f, "{}", self.robots.0.len()));

        let mut idx = Pos::zero();

        for row in self.board.chunks(self.size.to_usize().unwrap()) {
            for &b in row {
                if b {
                    try!(write!(f, "{}", WALL_CHAR));
                } else {
                    if let Some(robot) = self.robots.0.iter().position(|&pos| pos == idx) {
                        try!(write!(f, "{}", robot));
                    } else if idx == self.goal {
                        try!(write!(f, "{}", GOAL_CHAR));
                    } else {
                        try!(write!(f, "{}", EMPTY_CHAR));
                    }
                }

                idx = idx + Pos::one();
            }
            try!(writeln!(f, ""));
        }

        Ok(())
    }
}

/// Display directions by their initial letter
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use Direction::*;

        let s = match *self {
            Up => "U",
            Right => "R",
            Down => "D",
            Left => "L",
        };

        write!(f, "{}", s)
    }
}

/// Display moves by robot number and direction, e.g. "0L" or "1U"
impl<Pos> fmt::Display for Move<Pos> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", self.robot, self.dir)
    }
}

impl<Pos> Move<Pos> {
    /// Shorthand for making a new move
    fn new(robot: Robot, to: Pos, dir: Direction) -> Move<Pos> {
        Move {
            to: to,
            robot: robot,
            dir: dir,
        }
    }
}

impl Direction {
    /// What should be added to an index to move in the given direction on a board of size `size`
    fn delta_idx<Pos>(&self, size: Pos) -> i32
        where Pos: ToPrimitive,
    {
        use std::ops::Neg;
        use Direction::*;

        match *self {
            Up => (size.to_i32().unwrap()).neg(),
            Right => 1,
            Down => size.to_i32().unwrap(),
            Left => -1,
        }
    }

    fn is_opposite(&self, other: &Self) -> bool {
        use Direction::*;

        match *self {
            Up => *other == Down,
            Right => *other == Left,
            Down => *other == Up,
            Left => *other == Right,
        }
    }

}

impl<T, Pos, N> Board<T, Pos, N>
    where Pos: FromPrimitive + Integer + Copy,
          N: ArrayLength<Pos>,
{

    /// Is `i` a reachable index starting at `from` in the given direction without considering walls
    fn within_board(&self, i: i32, from: Pos, dir: &Direction) -> bool
    {
        use Direction::*;

        match *dir {
            Up => i >= 0,
            Right => (Pos::from_i32(i).unwrap()) < (from / self.size + Pos::one()) * self.size,
            Down => (i as usize) < self.board.len(),
            Left => (Pos::from_i32(i).unwrap()) >= (from / self.size) * self.size,
        }
    }
}

impl<Pos, N> Board<bool, Pos, N>
    where Pos: Integer + FromPrimitive + ToPrimitive + Copy,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
    /// Make an endpoint board from the current one
    /// This happens only once before the search starts, so it doesn't have to be very fast.
    fn to_endpoints_board(&self) -> Board<Endpoints<Pos>, Pos, N> {
        use Direction::*;

        let mut board: Vec<Endpoints<Pos>> = Vec::with_capacity(self.board.len());

        for idx in 0 .. self.board.len() {
            if self.board[idx] {
                // Endpoints from within a wall should be considered undefined
                board.push(Endpoints {
                    up: Pos::zero(),
                    right: Pos::zero(),
                    down: Pos::zero(),
                    left: Pos::zero(),
                });
            } else {
                let i = Pos::from_usize(idx).unwrap();

                let (left, right) = if i % self.size == Pos::zero() ||
                    self.board[idx - 1]
                {
                    (i, self.endpoint_in_direction(i, Right))
                } else {
                    let ref prev = board[idx - 1];
                    (prev.left, prev.right)
                };

                let (up, down) = if i < self.size ||
                    self.board[idx - self.size.to_usize().unwrap()]
                {
                    (i, self.endpoint_in_direction(i, Down))
                } else {
                    let ref prev = board[idx - self.size.to_usize().unwrap()];
                    (prev.up, prev.down)
                };

                board.push(Endpoints {
                    up: up,
                    right: right,
                    down: down,
                    left: left,
                })
            }
        }

        Board {
            board: board.into_boxed_slice(),
            robots: self.robots,
            size: self.size,
            goal: self.goal,
        }
    }

    /// What index is reachable from the given position in the given direction
    fn endpoint_in_direction(&self, from: Pos, dir: Direction) -> Pos {
        let mut to = from;
        let di = dir.delta_idx(self.size);

        let mut next = to.to_i32().unwrap() + di;
        while next >= 0 &&
            self.within_board(next, from, &dir) &&
            !self.board[next as usize]
        {
            to = Pos::from_i32(next).unwrap();
            next += di;
        }

        to
    }
}

impl<Pos, N> Board<Endpoints<Pos>, Pos, N>
    where Pos: Integer + ToPrimitive + Copy,
          N: ArrayLength<Pos>,
{

    /// Push all possible on the board given a specific placement of robots into `moves`
    /// This happens at each node of the search, so it should be as fast as possible.
    fn possible_moves(&self, robots: &Robots<Pos, N>, moves: &mut Vec<Move<Pos>>) {
        use Direction::*;

        // Loop through each robot's number and position
        for (i, &robot) in (0..self.robots.0.len() as u8).zip(robots.0.iter()) {
            // Look up the possible endpoints if no other robot is in the way
            let Endpoints {mut up, mut right, mut down, mut left} = self.board[robot.to_usize().unwrap()];

            // Check if there actually are robots in the way
            for &r in robots.0.iter() {
                if r > robot {
                    if r <= right {
                        right = r - Pos::one();
                    }

                    if r <= down && r % self.size == robot % self.size {
                        down = r - self.size;
                    }
                } else if r < robot {
                    if r >= left {
                        left = r + Pos::one();
                    }

                    if r >= up && r % self.size == robot % self.size {
                        up = r + self.size;
                    }
                }
            }

            // Push the found moves

            if up != robot {
                moves.push(Move::new(i, up, Up));
            }

            if right != robot {
                moves.push(Move::new(i, right, Right));
            }

            if down != robot {
                moves.push(Move::new(i, down, Down));
            }

            if left != robot {
                moves.push(Move::new(i, left, Left));
            }
        }
    }
}

/// Utility function for reading a file into a string
pub fn read_file(name: &Path) -> io::Result<String> {
    use std::fs::File;

    let mut f = try!(File::open(name));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).expect("Provide a map on stdin");

    let mut lines = buffer.lines();
    let size: usize = lines.next().unwrap().parse().unwrap();
    let log = (size as f64).log2().ceil() as usize;

    let robots: u8 = lines.next().unwrap().parse().unwrap();

    match log {
        1 | 2 | 3 | 4 => {
            pos_runner::<u8>(robots, &buffer);
        },
        5 | 6 | 7 | 8 => {
            pos_runner::<u16>(robots, &buffer);
        },
        _ => {
            pos_runner::<u32>(robots, &buffer);
        },
    }
}

fn pos_runner<Pos>(robots: u8, buffer: &str)
    where Pos: fmt::Debug + fmt::Display + FromStr + Integer + FromPrimitive + ToPrimitive + Copy + Hash + Default,
          Pos::Err: fmt::Debug,
{
    use typenum::{U1, U2, U3, U4, U5, U6, U7, U8, U9, U10};

    match robots {
        1 => runner(Board::<bool, Pos, U1>::from_str(&buffer).unwrap()),
        2 => runner(Board::<bool, Pos, U2>::from_str(&buffer).unwrap()),
        3 => runner(Board::<bool, Pos, U3>::from_str(&buffer).unwrap()),
        4 => runner(Board::<bool, Pos, U4>::from_str(&buffer).unwrap()),
        5 => runner(Board::<bool, Pos, U5>::from_str(&buffer).unwrap()),
        6 => runner(Board::<bool, Pos, U6>::from_str(&buffer).unwrap()),
        7 => runner(Board::<bool, Pos, U7>::from_str(&buffer).unwrap()),
        8 => runner(Board::<bool, Pos, U8>::from_str(&buffer).unwrap()),
        9 => runner(Board::<bool, Pos, U9>::from_str(&buffer).unwrap()),
        10 => runner(Board::<bool, Pos, U10>::from_str(&buffer).unwrap()),
        c => panic!("{} robots, only 1-10 is supported.", c),
    }
}

fn runner<Pos, N>(board: Board<bool, Pos, N>)
    where Pos: fmt::Display + Integer + FromPrimitive + ToPrimitive + Copy + Hash + Default,
          N: ArrayLength<Pos>,
          N::ArrayType: Copy,
{
    use std::mem::size_of;

    println!("Pos size: {} byte", size_of::<Pos>());
    println!("Robots array size: {}", N::to_u8());

    println!("{}", board);

    let endboard = board.to_endpoints_board();

    let sol = ids(&endboard);
    let steps = sol.len();

    for m in sol {
        println!("{}", m);
    }

    println!("Solved in {} steps.", steps);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use std::path::Path;
    use typenum::{U1, U2, U4, U8};

    const BOARDS_PATH: &'static str = "../../boards/random_boards/3_hard/";

    #[test]
    fn ricochet_test_1() {
        let path = Path::new(BOARDS_PATH)
            .join(Path::new("n=15___robots=1___possible_in_15_moves.txt"));
        let s = read_file(&path).unwrap();

        let board: Board<_, u8, U1> = Board::from_str(&s).unwrap().to_endpoints_board();
        let sol = ids(&board);

        assert_eq!(sol.len(), 10);
    }

    #[test]
    fn ricochet_test_2() {
        let path = Path::new(BOARDS_PATH)
            .join(Path::new("n=15___robots=2___possible_in_15_moves.txt"));
        let s = read_file(&path).unwrap();

        let board: Board<_, u8, U2> = Board::from_str(&s).unwrap().to_endpoints_board();
        let sol = ids(&board);

        assert_eq!(sol.len(), 8);
    }

    #[test]
    fn ricochet_test_4() {
        let path = Path::new(BOARDS_PATH)
            .join(Path::new("n=15___robots=4___possible_in_15_moves.txt"));
        let s = read_file(&path).unwrap();

        let board: Board<_, u8, U4> = Board::from_str(&s).unwrap().to_endpoints_board();
        let sol = ids(&board);

        assert_eq!(sol.len(), 6);
    }

    #[test]
    fn ricochet_test_8() {
        let path = Path::new(BOARDS_PATH)
            .join(Path::new("n=15___robots=8___possible_in_15_moves.txt"));
        let s = read_file(&path).unwrap();

        let board: Board<_, u8, U8> = Board::from_str(&s).unwrap().to_endpoints_board();
        let sol = ids(&board);

        assert_eq!(sol.len(), 5);
    }

}
