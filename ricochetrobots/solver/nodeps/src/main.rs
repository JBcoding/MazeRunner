use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher, BuildHasherDefault};
use std::io::{self, Read};
use std::ops::{Index, IndexMut, Range, Add, Sub, Mul, Div, Rem};
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;

type MyHasher = BuildHasherDefault<FnvHasher>;

const WALL_CHAR: char = '#';
const GOAL_CHAR: char = 'G';
const EMPTY_CHAR: char = ' ';

type Robot = u8;

pub trait Robots
    : Index<u8, Output=<Self as Robots>::Pos>
    + Index<Range<usize>>
    + IndexMut<u8>
    + Copy
    + Default
    + Eq
    + Ord
    + Hash
{
    type Pos : Position;

    fn symmetric(self) -> Self;

    fn contains(&self, &Self::Pos) -> bool;

    fn iter(&self) -> Iter<Self::Pos>;

    fn len(&self) -> usize;
}

pub trait Position
    : Sized
    + Copy
    + fmt::Display
    + Eq
    + Ord
    + Zero
    + One
    + Add<Output=Self>
    + Sub<Output=Self>
    + Mul<Output=Self>
    + Div<Output=Self>
    + Rem<Output=Self>
{
    fn to_usize(&self) -> usize;
    fn from_usize(usize) -> Self;

    fn to_i32(&self) -> i32;
    fn from_i32(i32) -> Self;
}

pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

/// Two intended types for T:
/// - bool: The raw board as read from input, true index has a wall
/// - Endpoints: What indices can be reached from each index in each direction, disregarding where the robots are placed
pub struct Board<T, R, Pos> {
    board: Box<[T]>,
    robots: R,
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
    Up    = 0b0001,
    Right = 0b0010,
    Down  = 0b0100,
    Left  = 0b1000,
}

/// Move a robot to an index and remember the direction for reconstructing the solution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move<Pos> {
    to: Pos,
    robot: Robot,
    dir: Direction,
}

/// Compact representation of a Move
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveLite(u8);

#[derive(Debug, Clone)]
pub struct Node<R> {
    state: R,
    move_idx: u32,
    path_cost: u16,
    estimated_cost: u16,
}

impl<R: Eq> PartialEq for Node<R> {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

impl<R: Eq> Eq for Node<R> {}

impl<R: Ord> PartialOrd for Node<R> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<R: Ord> Ord for Node<R> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimated_cost.cmp(&self.estimated_cost)
    }
}

fn solution(
    mut idx: usize,
    moves: &[MoveLite],
    moves_indices: &[u32],
) -> Vec<MoveLite> {
    let mut sol = Vec::new();

    let mut m = moves[idx];
    idx = moves_indices[idx] as usize;

    while idx > 0 {
        sol.push(m);
        m = moves[idx];
        idx = moves_indices[idx] as usize;
    }

    sol.push(m);

    sol.reverse();
    sol
}

fn child_node<R: Robots>(
    dists: &Vec<(u16, Vec<R::Pos>)>,
    parent: &Node<R>,
    action: Move<R::Pos>,
    move_idx: u32,
) -> Node<R> {
    let mut state = parent.state;
    state[action.robot] = action.to;

    let (dist, ref spots) = dists[state[0].to_usize()];

    let covered = spots
        .iter()
        .filter(|spot| state.contains(spot))
        .count();

    let uncovered = (spots.len() - covered) as u16;

    let path_cost = parent.path_cost + 1;

    Node {
        state: state,
        move_idx: move_idx,
        path_cost: path_cost,
        estimated_cost: path_cost + dist + uncovered,
    }
}

pub fn astar<R: Robots>(
    board: &Board<Endpoints<R::Pos>, R, R::Pos>,
    dists: &Vec<(u16, Vec<R::Pos>)>
) -> Option<Vec<MoveLite>>
{
    let initial = Node {
        state: board.robots,
        move_idx: 0,
        path_cost: 0,
        estimated_cost: 0,
    };

    let mut frontier = BinaryHeap::new();

    let mut explored: HashSet<_, MyHasher> = HashSet::default();

    let mut moves: Vec<MoveLite> = Vec::new();
    let mut moves_indices: Vec<u32> = Vec::new();

    moves.push(MoveLite::new(board.robots.len() as u8, Direction::Up));
    moves_indices.push(0);

    let mut possible_moves = Vec::with_capacity(4 * board.robots.len());

    frontier.push(initial);

    while let Some(node) = frontier.pop() {
        if node.state[0] == board.goal {
            return Some(solution(
                node.move_idx as usize, &moves, &moves_indices
            ));
        }

        possible_moves.clear();
        board.possible_moves(node.state, &mut possible_moves);

        for m in possible_moves.iter().cloned() {
            let child = child_node(dists, &node, m, moves.len() as u32);

            let child_symmetry = child.state.symmetric();

            if !explored.contains(&child_symmetry) {
                moves.push(MoveLite::new(m.robot, m.dir));
                moves_indices.push(node.move_idx);

                explored.insert(child_symmetry);
                frontier.push(child);
            }
        }
    }

    None
}

/// Parse a board from a string
impl<R: Robots> FromStr for Board<bool, R, R::Pos>
    where R::Pos: FromStr,
<R::Pos as FromStr>::Err: fmt::Debug,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let size: R::Pos = lines.next().unwrap().parse().unwrap();
        lines.next().unwrap();
        let mut robots = R::default();
        let mut goal = R::Pos::zero();

        let mut board = Vec::with_capacity((size * size).to_usize());
        let mut idx: R::Pos = R::Pos::zero();

        for line in lines {
            for c in line.chars() {
                match c {
                    EMPTY_CHAR => board.push(false),
                    WALL_CHAR => board.push(true),
                    d if d.is_digit(10) => {
                        let i = d.to_digit(10).unwrap();
                        robots[i as u8] = idx;
                        board.push(false);
                    },
                    GOAL_CHAR => {
                        goal = idx;
                        board.push(false);
                    },
                    c => return Err(format!("Unexpected character: {}", c)),
                }

                idx = idx + R::Pos::one();
            }
        }

        Ok(Board {
            board: board.into_boxed_slice(),
            robots: robots,
            size: size,
            goal: goal,
        })
    }
}

/// Display a board
impl<R: Robots> fmt::Display for Board<bool, R, R::Pos> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(writeln!(f, "{}", self.size));
        try!(writeln!(f, "{}", self.robots.len()));

        let mut idx = R::Pos::zero();

        for row in self.board.chunks(self.size.to_usize()) {
            for &b in row {
                if b {
                    try!(write!(f, "{}", WALL_CHAR));
                } else {
                    if let Some(robot) = self.robots
                        .iter()
                        // .take(self.robots_len as usize)
                        .position(|&pos| pos == idx)
                    {
                        try!(write!(f, "{}", robot));
                    } else if idx == self.goal {
                        try!(write!(f, "{}", GOAL_CHAR));
                    } else {
                        try!(write!(f, "{}", EMPTY_CHAR));
                    }
                }

                idx = idx + R::Pos::one();
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

impl fmt::Display for MoveLite {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", self.robot(), self.direction())
    }
}

impl<Pos> Move<Pos> {
    /// Shorthand for making a new move
    fn new(robot: Robot, to: Pos, dir: Direction) -> Self {
        Move {
            to: to,
            robot: robot,
            dir: dir,
        }
    }
}

impl MoveLite {
    fn new(robot: Robot, dir: Direction) -> Self {
        MoveLite(robot | ((dir as u8) << 4))
    }

    fn robot(&self) -> Robot {
        self.0 & 0b1111
    }

    fn direction(&self) -> Direction {
        use Direction::*;

        match self.0 >> 4 {
            0b0001 => Up,
            0b0010 => Right,
            0b0100 => Down,
            0b1000 => Left,
            _ => panic!("Wrong dir: {}", self.0),
        }
    }
}


impl Direction {
    /// What should be added to an index to move in the given direction on a board of size `size`
    fn delta_idx<R: Robots>(&self, size: R::Pos) -> i32 {
        use std::ops::Neg;
        use Direction::*;

        match *self {
            Up => (size.to_i32()).neg(),
            Right => 1,
            Down => size.to_i32(),
            Left => -1,
        }
    }
}

impl<T, R: Robots> Board<T, R, R::Pos> {

    /// Is `i` a reachable index starting at `from` in the given direction without considering walls
    fn within_board(&self, i: i32, from: R::Pos, dir: &Direction) -> bool
    {
        use Direction::*;

        match *dir {
            Up => i >= 0,
            Right => (R::Pos::from_i32(i)) < (from / self.size + R::Pos::one()) * self.size,
            Down => (i as usize) < self.board.len(),
            Left => (R::Pos::from_i32(i)) >= (from / self.size) * self.size,
        }
    }
}

impl<R: Robots> Board<bool, R, R::Pos> {
    /// Make an endpoint board from the current one
    /// This happens only once before the search starts, so it doesn't have to be very fast.
    fn to_endpoints_board(&self) -> Board<Endpoints<R::Pos>, R, R::Pos> {
        use Direction::*;

        let mut board = Vec::with_capacity(self.board.len());

        for idx in 0..self.board.len() {
            if self.board[idx] {
                // Endpoints from within a wall should be considered undefined
                board.push(Endpoints {
                    up: R::Pos::zero(),
                    right: R::Pos::zero(),
                    down: R::Pos::zero(),
                    left: R::Pos::zero(),
                });
            } else {
                let i = R::Pos::from_usize(idx);

                let (left, right) = if i % self.size == R::Pos::zero() ||
                    self.board[idx - 1]
                {
                    (i, self.endpoint_in_direction(i, Right))
                } else {
                    let ref prev = board[idx - 1];
                    (prev.left, prev.right)
                };

                let (up, down) = if i < self.size ||
                    self.board[idx - self.size.to_usize()]
                {
                    (i, self.endpoint_in_direction(i, Down))
                } else {
                    let ref prev = board[idx - self.size.to_usize()];
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
    fn endpoint_in_direction(&self, from: R::Pos, dir: Direction) -> R::Pos {
        let mut to = from;
        let di = dir.delta_idx::<R>(self.size);

        let mut next = to.to_i32() + di;
        while next >= 0 &&
            self.within_board(next, from, &dir) &&
            !self.board[next as usize]
        {
            to = R::Pos::from_i32(next);
            next += di;
        }

        to
    }

    fn get_dists(&self) -> Vec<(u16, Vec<R::Pos>)> {
        use Direction::*;
        use std::collections::VecDeque;

        let mut queue = VecDeque::new();

        queue.push_back((self.goal, 0, vec![]));

        let mut dists = vec![(std::u16::MAX, vec![]); self.board.len()];

        while let Some((from, dist, spots)) = queue.pop_front() {
            dists[from.to_usize()] = (dist, spots.clone());

            for dir in [Up, Right, Down, Left].into_iter() {
                let di = dir.delta_idx::<R>(self.size);

                let opposite = match *dir {
                    Up => Down,
                    Right => Left,
                    Down => Up,
                    Left => Right,
                };

                let mut new_spots = spots.clone();

                let prev = from.to_i32() - di;
                if prev >= 0
                    && (prev as usize) < self.board.len()
                    && !self.board[prev as usize]
                    && self.within_board(prev, from, &opposite)
                {
                    new_spots.push(R::Pos::from_i32(prev))
                }

                let mut next = from.to_i32() + di;

                while next >= 0
                    && self.within_board(next, from, dir)
                    && !self.board[next as usize]
                    && dists[next as usize].0 == std::u16::MAX
                {
                    let mut found = false;

                    for &mut (n, _, ref mut rs) in queue.iter_mut() {
                        if n.to_i32() == next {
                            found = true;

                            if new_spots.len() < rs.len() {
                                *rs = new_spots.clone();
                            }

                            break;
                        }
                    }

                    if !found {
                        queue.push_back(
                            (R::Pos::from_i32(next), dist+1, new_spots.clone())
                        );
                    }
                    next += di;
                }
            }
        }

        dists
    }
}

impl<R: Robots> Board<Endpoints<R::Pos>, R, R::Pos> {

    /// Push all possible on the board given a specific placement of robots into `moves`
    /// This happens at each node of the search, so it should be as fast as possible.
    fn possible_moves(&self, robots: R, moves: &mut Vec<Move<R::Pos>>) {
        use Direction::*;

        // Loop through each robot's number and position
        for (i, &robot) in (0..).zip(robots.iter()) {
            // Look up the possible endpoints if no other robot is in the way
            let Endpoints {mut up, mut right, mut down, mut left} = self.board[robot.to_usize()];

            // Check if there actually are robots in the way
            for (_, &r) in (0..).zip(robots.iter()) {
                if r > robot {
                    if r <= right {
                        right = r - R::Pos::one();
                    } else if r <= down && r % self.size == robot % self.size {
                        down = r - self.size;
                    }
                } else if r < robot {
                    if r >= left {
                        left = r + R::Pos::one();
                    } else if r >= up && r % self.size == robot % self.size {
                        up = r + self.size;
                    }
                }
            }

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
            match robots {
                1 => runner::<Robots1u8>(&buffer),
                2 => runner::<Robots2u8>(&buffer),
                3 => runner::<Robots3u8>(&buffer),
                4 => runner::<Robots4u8>(&buffer),
                5 => runner::<Robots5u8>(&buffer),
                6 => runner::<Robots6u8>(&buffer),
                7 => runner::<Robots7u8>(&buffer),
                8 => runner::<Robots8u8>(&buffer),
                9 => runner::<Robots9u8>(&buffer),
                10 => runner::<Robots10u8>(&buffer),
                x => panic!("ERROR: {} robots are not supported!", x),
            }
        },
        5 | 6 | 7 | 8 => {
            match robots {
                1 => runner::<Robots1u16>(&buffer),
                2 => runner::<Robots2u16>(&buffer),
                3 => runner::<Robots3u16>(&buffer),
                4 => runner::<Robots4u16>(&buffer),
                5 => runner::<Robots5u16>(&buffer),
                6 => runner::<Robots6u16>(&buffer),
                7 => runner::<Robots7u16>(&buffer),
                8 => runner::<Robots8u16>(&buffer),
                9 => runner::<Robots9u16>(&buffer),
                10 => runner::<Robots10u16>(&buffer),
                x => panic!("ERROR: {} robots are not supported!", x),
            }
        },
        _ => {
            match robots {
                1 => runner::<Robots1u32>(&buffer),
                2 => runner::<Robots2u32>(&buffer),
                3 => runner::<Robots3u32>(&buffer),
                4 => runner::<Robots4u32>(&buffer),
                5 => runner::<Robots5u32>(&buffer),
                6 => runner::<Robots6u32>(&buffer),
                7 => runner::<Robots7u32>(&buffer),
                8 => runner::<Robots8u32>(&buffer),
                9 => runner::<Robots9u32>(&buffer),
                10 => runner::<Robots10u32>(&buffer),
                x => panic!("ERROR: {} robots are not supported!", x),
            }
        },
    }
}

fn runner<R: Robots>(buffer: &str)
    where R::Pos: FromStr, <R::Pos as FromStr>::Err: fmt::Debug,
{
    let board = Board::<bool, R, _>::from_str(&buffer).unwrap();

    let dists = board.get_dists();
    let endboard = board.to_endpoints_board();

    let sol = astar(&endboard, &dists).expect("No solution");

    for m in sol.iter() {
        println!("{}", m);
    }

    // println!("Solved in {} steps.", sol.len());
}

macro_rules! struct_Robots {
    ($name:ident, $n: expr, $pos:ty) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name([$pos; $n]);
    }
}

macro_rules! impl_Robots_for {
    ($t:ty, $pos:ty) => {
        impl Index<u8> for $t {
            type Output = $pos;

            fn index<'a>(&'a self, idx: u8) -> &'a Self::Output {
                &self.0[idx as usize]
            }
        }

        impl Index<Range<usize>> for $t {
            type Output = [$pos];

            fn index<'a>(&'a self, idx: Range<usize>) -> &'a Self::Output {
                &self.0[idx]
            }
        }

        impl IndexMut<u8> for $t {
            fn index_mut<'a>(&'a mut self, idx: u8) -> &'a mut $pos {
                &mut self.0[idx as usize]
            }
        }

        impl Robots for $t {
            type Pos = $pos;

            fn symmetric(mut self) -> Self {
                &mut self.0[1..].sort();
                self
            }

            fn contains(&self, p: &Self::Pos) -> bool {
                self.0.contains(p)
            }

            fn iter(&self) -> Iter<Self::Pos> {
                self.0.iter()
            }

            fn len(&self) -> usize {
                self.0.len()
            }

        }
    }
}

macro_rules! impl_Position_for {
    ($t:ty) => {
        impl Zero for $t {
            fn zero() -> Self {
                0
            }
        }

        impl One for $t {
            fn one() -> Self {
                1
            }
        }

        impl Position for $t {
            fn to_usize(&self) -> usize {
                *self as usize
            }

            fn from_usize(x: usize) -> Self {
                x as Self
            }

            fn to_i32(&self) -> i32 {
                *self as i32
            }

            fn from_i32(x: i32) -> Self {
                x as Self
            }
        }
    }
}

impl_Position_for!(u8);
impl_Position_for!(u16);
impl_Position_for!(u32);


struct_Robots!(Robots1u8 , 1, u8);
struct_Robots!(Robots1u16, 1, u16);
struct_Robots!(Robots1u32, 1, u32);

struct_Robots!(Robots2u8 , 2, u8);
struct_Robots!(Robots2u16, 2, u16);
struct_Robots!(Robots2u32, 2, u32);

struct_Robots!(Robots3u8 , 3, u8);
struct_Robots!(Robots3u16, 3, u16);
struct_Robots!(Robots3u32, 3, u32);

struct_Robots!(Robots4u8 , 4, u8);
struct_Robots!(Robots4u16, 4, u16);
struct_Robots!(Robots4u32, 4, u32);

struct_Robots!(Robots5u8 , 5, u8);
struct_Robots!(Robots5u16, 5, u16);
struct_Robots!(Robots5u32, 5, u32);

struct_Robots!(Robots6u8 , 6, u8);
struct_Robots!(Robots6u16, 6, u16);
struct_Robots!(Robots6u32, 6, u32);

struct_Robots!(Robots7u8 , 7, u8);
struct_Robots!(Robots7u16, 7, u16);
struct_Robots!(Robots7u32, 7, u32);

struct_Robots!(Robots8u8 , 8, u8);
struct_Robots!(Robots8u16, 8, u16);
struct_Robots!(Robots8u32, 8, u32);

struct_Robots!(Robots9u8 , 9, u8);
struct_Robots!(Robots9u16, 9, u16);
struct_Robots!(Robots9u32, 9, u32);

struct_Robots!(Robots10u8 , 10, u8);
struct_Robots!(Robots10u16, 10, u16);
struct_Robots!(Robots10u32, 10, u32);


impl_Robots_for!(Robots1u8 , u8);
impl_Robots_for!(Robots1u16, u16);
impl_Robots_for!(Robots1u32, u32);

impl_Robots_for!(Robots2u8 , u8);
impl_Robots_for!(Robots2u16, u16);
impl_Robots_for!(Robots2u32, u32);

impl_Robots_for!(Robots3u8 , u8);
impl_Robots_for!(Robots3u16, u16);
impl_Robots_for!(Robots3u32, u32);

impl_Robots_for!(Robots4u8 , u8);
impl_Robots_for!(Robots4u16, u16);
impl_Robots_for!(Robots4u32, u32);

impl_Robots_for!(Robots5u8 , u8);
impl_Robots_for!(Robots5u16, u16);
impl_Robots_for!(Robots5u32, u32);

impl_Robots_for!(Robots6u8 , u8);
impl_Robots_for!(Robots6u16, u16);
impl_Robots_for!(Robots6u32, u32);

impl_Robots_for!(Robots7u8 , u8);
impl_Robots_for!(Robots7u16, u16);
impl_Robots_for!(Robots7u32, u32);

impl_Robots_for!(Robots8u8 , u8);
impl_Robots_for!(Robots8u16, u16);
impl_Robots_for!(Robots8u32, u32);

impl_Robots_for!(Robots9u8 , u8);
impl_Robots_for!(Robots9u16, u16);
impl_Robots_for!(Robots9u32, u32);

impl_Robots_for!(Robots10u8 , u8);
impl_Robots_for!(Robots10u16, u16);
impl_Robots_for!(Robots10u32, u32);

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::str::FromStr;
    use std::path::Path;

    const PATH_3_HARD: &'static str = "../../boards/random_boards/3_hard/";
    const PATH_5_VERY_LARGE: &'static str = "../../boards/random_boards/5_very_large/";

    fn tester<R: Robots>(dir: &str, board: &str, expected_steps: usize)
        where R::Pos: FromStr, <R::Pos as FromStr>::Err: fmt::Debug,
    {
        let path = Path::new(dir).join(Path::new(board));
        let s = read_file(&path).unwrap();

        let board: Board<bool, R, R::Pos> = Board::from_str(&s).unwrap();
        let endpoints = board.to_endpoints_board();
        let dists = board.get_dists();

        let sol = astar(&endpoints, &dists).expect("No solution");

        assert_eq!(sol.len(), expected_steps);
    }

    #[test]
    fn astar_15_1_15() {
        tester::<Robots1u8>
            (PATH_3_HARD, "n=15___robots=1___possible_in_15_moves.txt", 10);
    }

    #[test]
    fn astar_15_2_15() {
        tester::<Robots2u8>
            (PATH_3_HARD, "n=15___robots=2___possible_in_15_moves.txt", 8);
    }

    #[test]
    fn astar_15_4_15() {
        tester::<Robots4u8>
            (PATH_3_HARD, "n=15___robots=4___possible_in_15_moves.txt", 6);
    }

    #[test]
    fn astar_100_4_20() {
        tester::<Robots4u16>
            (PATH_5_VERY_LARGE, "n=100___robots=4___possible_in_20_moves.txt", 7);
    }
}

/// Verbatim from: https://github.com/servo/rust-fnv
pub struct FnvHasher(u64);

impl Default for FnvHasher {

    #[inline]
    fn default() -> FnvHasher {
        FnvHasher(0xcbf29ce484222325)
    }
}

impl Hasher for FnvHasher {

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let FnvHasher(mut hash) = *self;

        for byte in bytes.iter() {
            hash = hash ^ (*byte as u64);
            hash = hash.wrapping_mul(0x100000001b3);
        }

        *self = FnvHasher(hash);
    }
}
