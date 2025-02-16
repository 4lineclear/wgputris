use std::time::Instant;
use std::u8;

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
enum Action {
    MinoAction(MinoAction),
    Idle,
}

#[derive(Debug)]
enum MinoAction {
    Horizontal(i8),
    Vertical(i8),
}

#[derive(Debug, Clone)]
struct GameTime {
    // variable user timings
    right: Timings,
    left: Timings,
    down: Timings,
    // variable system timings
    gravity: u16,
    // other timings
    start: Instant,
    now: Instant,
}

#[derive(Debug, Default, Clone)]
struct Timings {
    das_limit: u16,
    arr_limit: u16,
    das: u16,
    arr: u16,
}

#[derive(Debug)]
struct MinoBag {
    is_held: bool,
    held: Option<Block>,
    minos: Vec<Block>,
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
    Left,
    Down,
    Right,
}

#[derive(Debug, Clone)]
pub struct BlockIter<'a> {
    ghost_points: [Point; 4],
    mino_points: [Point; 4],
    mino_block: Block,
    block: &'a [Option<Block>],
    index: u8,
    y: u8,
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

    pub fn hold(&mut self) {
        if self.bag.is_held {
            return;
        }
        self.bag.is_held = true;
        let old = self.bag.held.replace(self.mino.block);
        self.mino = old
            .map(Mino::new)
            .unwrap_or_else(|| self.bag.gen_mino(&mut self.rng));
    }

    pub fn place(&mut self) {
        self.bag.is_held = false;
        while self.move_mino(MinoAction::Vertical(1)) {}
        let old = self.mino;
        self.mino = self.bag.gen_mino(&mut self.rng);

        old.real_points().into_iter().for_each(|point| {
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

    pub fn rotate(&mut self, left: Option<bool>) {
        use Ori::*;
        let old = self.mino.points;
        let ori = match left {
            Some(false) => match self.mino.ori {
                Up => Right,
                Left => Up,
                Down => Left,
                Right => Down,
            },
            Some(true) => match self.mino.ori {
                Up => Left,
                Left => Down,
                Down => Right,
                Right => Up,
            },
            None => match self.mino.ori {
                Up => Down,
                Left => Right,
                Down => Up,
                Right => Left,
            },
        };
        self.mino.points = self.mino.block.points(ori);

        if self
            .mino
            .real_points()
            .iter()
            .all(|&p| self.board.check_block(p))
        {
            self.mino.ori = ori;
        } else {
            self.mino.points = old;
        }
    }

    fn try_move_mino(&mut self, mino: Mino, dx: i8, dy: i8) -> IPoint {
        let mapp =
            |point: IPoint| IPoint::new(point.x.saturating_add(dx), point.y.saturating_add(dy));
        if mino
            .real_points()
            .iter()
            .all(|&p| self.board.icheck_block(mapp(p.into())))
        {
            mapp(mino.pos)
        } else {
            mino.pos
        }
    }

    fn move_mino(&mut self, ma: MinoAction) -> bool {
        use MinoAction::*;

        let prev = self.mino;

        self.mino.pos = match ma {
            Horizontal(x) => self.try_move_mino(self.mino, x, 0),
            Vertical(y) => self.try_move_mino(self.mino, 0, y),
        };

        prev.pos != self.mino.pos || self.calc_ghost()
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

    pub fn move_x(&mut self, left: bool) {
        self.move_mino(MinoAction::Horizontal(if left { -1 } else { 1 }));
    }

    pub fn move_down(&mut self, amount: i8) {
        self.move_mino(MinoAction::Vertical(amount));
    }

    pub fn tick(&mut self) -> bool {
        use Action::*;

        (match self.time.tick() {
            MinoAction(ma) => self.move_mino(ma),
            Idle => false,
        }) || self.calc_ghost()
    }

    pub fn start(&mut self) {
        self.rng = Xoshiro256Plus::seed_from_u64(self.seed);
        self.time.start = Instant::now();
    }

    pub fn block_iter<'a>(&'a self, y: u8) -> BlockIter<'a> {
        BlockIter::new(y, &self.board.0[y as usize].0, self.mino, self.ghost)
    }
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
        return self.0.len() > p.y as usize
            && self.line(0).0.len() > p.x as usize
            && self.block(p).is_none();
    }
    pub fn icheck_block(&self, p: IPoint) -> bool {
        return p.x >= 0
            && p.y >= 0
            && self.0.len() > p.y as usize
            && self.line(0).0.len() > p.x as usize
            && self.block(p).is_none();
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
        Mino::new(self.next_mino(rng))
    }

    fn next_mino(&mut self, rng: &mut Xoshiro256Plus) -> Block {
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
            // das_limit: 100,
            right: Timings::default(),
            left: Timings::default(),
            down: Timings::default(),
            start: now,
            now,
            gravity: 0,
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
        timings.das = 0;
        timings.arr = 0;
    }

    fn count_move(&mut self, left: Option<bool>, time: u16) -> u8 {
        let timings = self.timings(left);
        timings.das += time;
        if timings.das < timings.das_limit {
            timings.arr = 0;
            return (timings.das == time) as u8;
        }
        timings.arr += time;

        let amount = timings.arr / timings.arr_limit;
        timings.arr %= timings.arr_limit;
        u8::try_from(amount).unwrap_or(u8::MAX)
    }

    pub fn tick(&mut self) -> Action {
        let now = Instant::now();
        let diff = now - self.now;
        self.now = now;
        self.gravity += diff.as_millis() as u16;

        if self.gravity > 1_600 {
            let mut drop = 0;
            while self.gravity > 1_600 {
                self.gravity -= 1_600;
                drop += 1;
            }
            Action::MinoAction(MinoAction::Vertical(drop))
        } else {
            Action::Idle
        }
    }
}

impl Block {
    fn points(self, ori: Ori) -> [Point; 4] {
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
    fn real_points(self) -> [Point; 4] {
        self.points.map(|p| p + self.pos)
    }
}

impl<'a> BlockIter<'a> {
    fn new(y: u8, block: &'a [Option<Block>], mino: Mino, ghost: Mino) -> Self {
        Self {
            ghost_points: ghost.real_points(),
            mino_points: mino.real_points(),
            mino_block: mino.block,
            block,
            index: 0,
            y,
        }
    }
    fn block_or_mino(&self) -> Option<(Block, bool)> {
        if self.mino_points.contains(&(self.index, self.y).into()) {
            return Some((self.mino_block, false));
        }
        if self.ghost_points.contains(&(self.index, self.y).into()) {
            return Some((self.mino_block, true));
        }
        self.block[self.index as usize].zip(Some(false))
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = Option<(Block, bool)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index as usize >= self.block.len() {
            return None;
        }
        let n = self.block_or_mino();
        self.index += 1;
        Some(n)
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

pub const I_WALLKICKS: [[IPoint; 5]; 8] = points![
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
