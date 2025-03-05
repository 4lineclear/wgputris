use std::time::Instant;

use rand::{seq::SliceRandom, RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;

pub mod point;

pub use point::IPoint;
pub use point::Point;

#[derive(Debug)]
pub struct Game {
    seed: u64,
    rng: Xoshiro256Plus,
    bag: MinoBag,
    mino: Mino,
    ghost: Mino,
    time: GameTime,
    board: Board,
}

#[derive(Debug)]
enum TimeAction {
    Drop(i8),
    Idle,
}

#[derive(Debug, Clone)]
struct GameTime {
    // variable user timings
    right: Timings,
    left: Timings,
    down: Timings,
    hard_drop: HardDrop,
    // variable system timings
    gravity: u32,
    grav_goal: u32,
    ticks: u32,
    // other timings
    start: Instant,
    now: Instant,
}

#[derive(Debug, Clone)]
struct HardDrop {
    scheduled: bool,
    goal: u32,
}

#[derive(Debug, Default, Clone)]
#[allow(unused)]
struct Timings {
    das: Ticker,
    arr: Ticker,
}

impl Timings {
    fn new(das_limit: u16, arr_limit: u16) -> Self {
        Self {
            das: Ticker::new(das_limit),
            arr: Ticker::new(arr_limit),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Ticker {
    pub goal: u16,
    pub value: u16,
}

#[derive(Debug)]
pub struct MinoBag {
    pub is_held: bool,
    pub held: Option<Block>,
    pub minos: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mino {
    pub ori: Ori,
    pub pos: IPoint,
    pub block: Block,
    pub points: [Point; 4],
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Ori {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

/// A single block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// cyan
    I,
    /// blue
    J,
    /// orange
    L,
    /// yellow
    O,
    /// green
    S,
    /// magenta
    T,
    /// red
    Z,
}

impl Game {
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or_else(|| rand::rng().next_u64());
        let mut rng = Xoshiro256Plus::seed_from_u64(seed);
        let mut bag = MinoBag::new(&mut rng);

        let mino = bag.gen_mino(&mut rng);
        let now = Instant::now();
        Self {
            seed,
            rng,
            bag,
            mino,
            ghost: mino,
            time: GameTime::new(now),
            board: Board::default(),
        }
    }

    fn hold(&mut self) {
        if self.bag.is_held {
            return;
        }
        self.bag.is_held = true;
        let old = self.bag.held.replace(self.mino.block);
        self.mino = old
            .map(Mino::new)
            .unwrap_or_else(|| self.bag.gen_mino(&mut self.rng));
    }

    fn hard_drop(&mut self) {
        self.bag.is_held = false;
        while self.move_mino(1, true) {}
        let old = self.mino;
        self.mino = self.bag.gen_mino(&mut self.rng);

        old.real_points().into_iter().flatten().for_each(|point| {
            *self.board.block_mut(point) = Some(old.block);
        });

        let mut line = self.board.lines().len();
        while line > 0 {
            let end = line;
            line -= 1;
            while self.board.line(line).blocks().iter().all(Option::is_some) {
                line -= 1;
            }
            let diff = end - line - 1;
            if diff != 0 {
                self.board.0[..end].rotate_right(diff);
                self.board.0[..diff]
                    .iter_mut()
                    .for_each(|l| *l = Line::default());
            }
        }
    }

    fn rotate(&mut self, left: Option<bool>) {
        use Ori::*;
        let ori = match left {
            Some(false) => match self.mino.ori {
                Up => Left,
                Right => Up,
                Down => Right,
                Left => Down,
            },
            Some(true) => match self.mino.ori {
                Up => Right,
                Right => Down,
                Down => Left,
                Left => Up,
            },
            None => match self.mino.ori {
                Up => Down,
                Right => Left,
                Down => Up,
                Left => Right,
            },
        };
        let new = Mino {
            ori,
            points: self.mino.block.points(ori),
            ..self.mino
        };

        if let Some(pos) = self.try_rotate(new, left.is_none()) {
            self.mino = new;
            self.mino.pos = pos;
        }
    }

    // maybe the ugliest code ever
    fn try_rotate(&self, mut mino: Mino, is_180: bool) -> Option<IPoint> {
        let from = self.mino.ori;
        let tests = match (mino.block, is_180) {
            (Block::I, _) => ori_code(from, mino.ori).map(|code| WALLKICKS_I[code].iter()),
            (_, false) => ori_code(from, mino.ori).map(|code| WALLKICKS[code].iter()),
            _ => ori_code_180(from, mino.ori).map(|code| WALLKICKS_180[code].iter()),
        };
        let orig_pos = mino.pos;
        for &test in tests? {
            mino.pos = orig_pos + test;
            if mino.check_points(|p| self.board.check_block(p)) {
                return Some(mino.pos);
            }
        }
        None
    }

    fn move_mino(&mut self, amount: i8, vert: bool) -> bool {
        let prev = self.mino;

        self.mino.pos = if vert {
            self.try_move_mino(self.mino, 0, amount)
        } else {
            self.try_move_mino(self.mino, amount, 0)
        };

        prev.pos != self.mino.pos || self.calc_ghost()
    }

    fn try_move_mino(&mut self, mino: Mino, dx: i8, dy: i8) -> IPoint {
        let mapp =
            |point: IPoint| IPoint::new(point.x.saturating_add(dx), point.y.saturating_add(dy));
        if mino.check_points(|p| self.board.icheck_block(mapp(p.into()))) {
            mapp(mino.pos)
        } else {
            mino.pos
        }
    }

    fn calc_ghost(&mut self) -> bool {
        let mut new = self.mino;
        loop {
            let ngpos = self.try_move_mino(new, 0, 1);
            if ngpos == new.pos {
                break;
            }
            new.pos = ngpos;
        }
        if self.ghost == new {
            false
        } else {
            self.ghost = new;
            true
        }
    }

    fn multi_move(&mut self, left: Option<bool>) {
        let amount = self.time.count_move(left);
        for _ in 0..amount {
            self.move_dir(left);
        }
    }

    fn move_dir(&mut self, left: Option<bool>) {
        if let Some(left) = left {
            self.move_mino(if left { -1 } else { 1 }, false);
        } else {
            self.move_mino(1, true);
        }
    }

    pub fn apply_action(&mut self, action: super::Action, pressed: bool) {
        use super::Action::*;
        if pressed {
            match action {
                Hold => self.hold(),
                Place => self.hard_drop(),
                Rotate180 => self.rotate(None),
                RotateLeft => self.rotate(Some(true)),
                RotateRight => self.rotate(Some(false)),
                MoveRight => self.multi_move(Some(false)),
                MoveLeft => self.multi_move(Some(true)),
                MoveDown => self.multi_move(None),
                Exit => (),
            }
        } else {
            match action {
                Hold => (),
                Place => (),
                Rotate180 => (),
                RotateLeft => (),
                RotateRight => (),
                MoveRight => self.time.reset_timing(Some(false)),
                MoveLeft => self.time.reset_timing(Some(true)),
                MoveDown => self.time.reset_timing(None),
                Exit => (),
            }
        }
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        use TimeAction::*;
        if self.try_move_mino(self.mino, 1, 0) != self.mino.pos {
            if self.time.hard_drop.increment(self.mino.pos.y) {
                self.hard_drop();
            }
        }

        (match self.time.tick(now) {
            Drop(amount) => self.move_mino(amount, true),
            Idle => false,
        }) || self.calc_ghost()
    }

    pub fn start(&mut self) {
        self.rng = Xoshiro256Plus::seed_from_u64(self.seed);
        self.time.start = Instant::now();
    }

    pub fn blocks(&self, y: u8) -> impl Iterator<Item = Option<Block>> + '_ {
        self.board.0[y as usize].blocks().iter().copied()
    }
    pub fn mino(&self) -> Mino {
        self.mino
    }
    pub fn ghost(&self) -> Mino {
        self.ghost
    }
    pub fn bag(&self) -> &MinoBag {
        &self.bag
    }
}

fn ori_code(from: Ori, to: Ori) -> Option<usize> {
    Some(match (from, to) {
        // 01, 10, 12, 21, 23, 32, 30, 03
        (Ori::Up, Ori::Right) => 0,
        (Ori::Right, Ori::Up) => 1,

        (Ori::Right, Ori::Down) => 2,
        (Ori::Down, Ori::Right) => 3,

        (Ori::Down, Ori::Left) => 4,
        (Ori::Left, Ori::Down) => 5,

        (Ori::Left, Ori::Up) => 6,
        (Ori::Up, Ori::Left) => 7,
        _ => return None,
    })
}

fn ori_code_180(from: Ori, to: Ori) -> Option<usize> {
    Some(match (from, to) {
        // 02, 20, 13, 31;
        (Ori::Up, Ori::Down) => 0,
        (Ori::Down, Ori::Up) => 1,

        (Ori::Right, Ori::Left) => 2,
        (Ori::Left, Ori::Right) => 3,
        _ => return None,
    })
}

impl Default for Game {
    fn default() -> Self {
        Self::new(None)
    }
}

/// The main board
///
/// Higher `y` is lower on the board
#[derive(Debug, Default, Clone, Copy)]
pub struct Board([Line; BOARD_HEIGHT as usize]);

pub const TOTAL_BLOCKS: u8 = BOARD_HEIGHT * BOARD_WIDTH;
pub const VISIBLE_START: u8 = 4;
pub const BOARD_VISIBLE_HEIGHT: u8 = BOARD_HEIGHT - VISIBLE_START;
pub const BOARD_HEIGHT: u8 = 24;
pub const BOARD_WIDTH: u8 = 10;

impl Board {
    pub fn lines(&self) -> &[Line] {
        &self.0
    }
    /// Returns the visible lines
    pub fn visible(&self) -> &[Line] {
        &self.0[4..]
    }
    pub fn origin(&self) -> Point {
        Point::new(0, 23)
    }
    pub fn line(&self, y: usize) -> Line {
        self.0[y]
    }
    pub fn block(&self, point: impl Into<Point>) -> Option<Block> {
        let Point { x, y } = point.into();
        self.line(y as usize).block(x)
    }
    fn block_mut(&mut self, point: impl Into<Point>) -> &mut Option<Block> {
        let Point { x, y } = point.into();
        self.0[y as usize].block_mut(x)
    }
    pub fn check_block(&self, p: Point) -> bool {
        self.0.len() > p.y as usize
            && self.line(0).0.len() > p.x as usize
            && self.block(p).is_none()
    }
    pub fn icheck_block(&self, p: IPoint) -> bool {
        p.x >= 0
            && p.y >= 0
            && self.0.len() > p.y as usize
            && self.line(0).0.len() > p.x as usize
            && self.block(p).is_none()
    }
}

/// A single line
#[derive(Debug, Default, Clone, Copy)]
pub struct Line([Option<Block>; BOARD_WIDTH as usize]);

impl Line {
    pub fn blocks(&self) -> &[Option<Block>] {
        &self.0
    }
    pub fn block(&self, x: impl Into<usize>) -> Option<Block> {
        self.0[x.into()]
    }
    fn block_mut(&mut self, x: impl Into<usize>) -> &mut Option<Block> {
        &mut self.0[x.into()]
    }
}

impl MinoBag {
    fn new(rng: &mut Xoshiro256Plus) -> Self {
        Self {
            is_held: false,
            held: None,
            minos: [random_minos(rng), random_minos(rng)].concat(),
        }
    }

    fn gen_mino(&mut self, rng: &mut Xoshiro256Plus) -> Mino {
        Mino::new(self.next_block(rng))
    }

    fn next_block(&mut self, rng: &mut Xoshiro256Plus) -> Block {
        let block = self.minos.remove(0);
        if self.minos.len() <= 7 {
            self.minos.extend(random_minos(rng));
        }
        block
    }
}

fn random_minos(rng: &mut Xoshiro256Plus) -> [Block; 7] {
    use Block::*;
    let mut minos = [I, T, O, L, J, S, Z];
    minos.shuffle(rng);
    minos
}

impl GameTime {
    fn new(now: Instant) -> Self {
        Self {
            right: Timings::new(12, 0),
            left: Timings::new(12, 0),
            down: Timings::new(0, 0),
            hard_drop: HardDrop::new(),
            start: now,
            now,
            gravity: 120,
            grav_goal: 120,
            ticks: 0,
        }
    }

    fn timings(&mut self, left: Option<bool>) -> &mut Timings {
        match left {
            Some(true) => &mut self.left,
            Some(false) => &mut self.right,
            None => &mut self.down,
        }
    }

    fn reset_timing(&mut self, left: Option<bool>) {
        let timings = self.timings(left);
        timings.das.reset();
        timings.arr.reset();
    }

    // TODO: fix move counting.

    // TODO: consider copying jxtris.
    fn count_move(&mut self, left: Option<bool>) -> u8 {
        if let Some(left) = left {
            self.reset_timing(Some(!left));
        }
        let timings = self.timings(left);
        timings.das.tick();
        if !timings.das.reached() {
            timings.arr.reset();
            return (timings.das.value == 1) as u8;
        }
        timings.arr.tick();

        let amount;
        if timings.arr.goal == 0 {
            amount = u8::MAX as u16;
            timings.arr.value = 0;
        } else {
            amount = timings.arr.value / timings.arr.goal;
            timings.arr.value %= timings.arr.goal;
        }
        u8::try_from(amount).unwrap_or(u8::MAX)
    }

    pub fn tick(&mut self, now: Instant) -> TimeAction {
        self.now = now;
        self.ticks += 1;
        self.grav_goal += 1;
        if self.grav_goal >= self.gravity {
            let mut drop = 0;
            while self.grav_goal >= self.gravity {
                self.grav_goal -= self.gravity;
                drop += 1;
            }
            TimeAction::Drop(drop)
        } else {
            TimeAction::Idle
        }
    }
}

impl Ticker {
    pub fn new(goal: u16) -> Self {
        Self {
            goal,
            ..Default::default()
        }
    }

    fn tick(&mut self) {
        self.value += 1;
    }
    fn reset(&mut self) {
        self.value = 0;
    }
    fn reached(&mut self) -> bool {
        self.value >= self.goal
    }
}

impl HardDrop {
    fn new() -> Self {
        Self {
            goal: 120 * 20,
            scheduled: false,
        }
    }
    fn increment(&mut self, y: i8) -> bool {
        if !self.scheduled {
            self.scheduled = true;
            self.reset_goal(y);
        }
        self.goal -= 1;
        if self.goal == 0 {
            self.reset_goal(0);
            true
        } else {
            false
        }
    }
    fn reset_goal(&mut self, y: i8) {
        self.goal = 120 * 24u8.saturating_sub(y.unsigned_abs()) as u32;
        self.scheduled = false;
    }
}

impl Block {
    pub fn points(self, ori: Ori) -> [Point; 4] {
        MINO_POINTS[self as usize][ori as usize]
    }
}

impl Mino {
    fn new(block: Block) -> Self {
        Mino {
            ori: Ori::Up,
            block,
            pos: (3, 3).into(),
            points: block.points(Ori::Up),
        }
    }
    pub fn real_points(self) -> Option<[Point; 4]> {
        point::maybe_cast_points(self.points.map(|p| p + self.pos))
    }
    fn check_points(&self, check: impl Fn(Point) -> bool) -> bool {
        self.real_points()
            .is_some_and(|points| points.into_iter().all(check))
    }
}

macro_rules! points {
    ($point:expr,
      $([
        $(($x:expr, $y:expr)),*
      ]),* $(,)?
    ) => {
        [$(
            [$(
                $point($x, $y),
            )*],
        )*]
    };
    ($point:expr,$([
      $([
        $(($x:expr, $y:expr)),*
      ]),* $(,)?
    ]),* $(,)? ) => {
        [$(
            [$(
                [$(
                    $point($x, $y),
                )*],
            )*],
        )*]
    };
}

const MINO_POINTS: [[[Point; 4]; 4]; 7] = points![
    Point::new,
    [
        [(0, 1), (1, 1), (2, 1), (3, 1)],
        [(1, 0), (1, 1), (1, 2), (1, 3)],
        [(0, 2), (1, 2), (2, 2), (3, 2)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
    ], // I
    [
        [(0, 1), (1, 1), (2, 1), (0, 0)],
        [(1, 0), (1, 1), (1, 2), (0, 2)],
        [(0, 1), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (1, 1), (1, 2), (2, 0)],
    ], // J
    [
        [(0, 1), (1, 1), (2, 1), (2, 0)],
        [(1, 0), (1, 1), (1, 2), (0, 0)],
        [(0, 1), (1, 1), (2, 1), (0, 2)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
    ], // L
    [
        [(1, 0), (1, 1), (2, 0), (2, 1)],
        [(1, 0), (1, 1), (2, 0), (2, 1)],
        [(1, 0), (1, 1), (2, 0), (2, 1)],
        [(1, 0), (1, 1), (2, 0), (2, 1)],
    ], // O
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(0, 0), (0, 1), (1, 1), (1, 2)],
        [(1, 1), (2, 1), (0, 2), (1, 2)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
    ], // S
    [
        [(1, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
        [(0, 1), (1, 1), (2, 1), (1, 2)],
        [(1, 0), (1, 1), (2, 1), (1, 2)],
    ], // T
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(1, 0), (0, 1), (1, 1), (0, 2)],
        [(0, 1), (1, 1), (1, 2), (2, 2)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
    ], // Z
];

pub const WALLKICKS: [[IPoint; 5]; 8] = points![
    IPoint::new,
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
];

pub const WALLKICKS_I: [[IPoint; 5]; 8] = points![
    IPoint::new,
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
];

pub const WALLKICKS_180: [[IPoint; 6]; 4] = points![
    IPoint::new,
    [(0, 0), (0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)],
    [(0, 0), (0, 1), (1, 1), (-1, 1), (1, 0), (-1, 0)],
    [(0, 0), (-1, 0), (-1, -2), (-1, -1), (0, -2), (0, -1)],
    [(0, 0), (1, 0), (1, -2), (1, -1), (0, -2), (0, -1)],
];

// fn check_180(mino: Mino, ori_to: Ori, b: &Board) -> bool {
//     todo!()
// }
