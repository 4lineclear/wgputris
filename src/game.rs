use std::time::Duration;
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
    start: Instant,
    now: Instant,
    drop: Duration,
}

#[derive(Debug)]
struct MinoBag {
    held: Option<Block>,
    minos: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mino {
    pub ori: Ori,
    pub pos: Point,
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
    /// magenta
    T,
    /// yellow
    O,
    /// orange
    L,
    /// blue
    J,
    /// green
    S,
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
            time: GameTime::new(now),
            board: Board::default(),
        }
    }

    fn apply_action(&mut self, action: Action) -> bool {
        use Action::*;
        match action {
            MinoAction(ma) => self.move_mino(ma),
            Idle => false,
        }
    }

    fn move_mino(&mut self, ma: MinoAction) -> bool {
        use MinoAction::*;

        let prev = self.mino;

        let mut try_move = |dx, dy| {
            let mapp = |point: Point| {
                Point::new(
                    point.x.saturating_add_signed(dx),
                    point.y.saturating_add_signed(dy),
                )
            };
            if self
                .mino
                .real_points()
                .iter()
                .all(|&p| (|orig| self.board().check_block(mapp(orig)))(p))
            {
                self.mino.pos = mapp(self.mino.pos);
            }
        };

        match ma {
            Horizontal(x) => try_move(x, 0),
            Vertical(y) => try_move(0, y),
        }

        prev != self.mino
    }

    pub fn place(&mut self) {
        let old = self.mino;
        self.mino = self.bag.gen_mino(&mut self.rng);

        old.real_points().into_iter().for_each(|point| {
            *self.board.block_mut(point) = Some(old.block);
        });
    }

    pub fn move_x(&mut self, amount: i8) {
        self.move_mino(MinoAction::Horizontal(amount));
    }

    pub fn move_down(&mut self, amount: i8) {
        self.move_mino(MinoAction::Vertical(amount));
    }

    pub fn tick(&mut self) -> bool {
        let action = self.time.tick();
        self.apply_action(action)
    }

    pub fn start(&mut self) {
        self.rng = Xoshiro256Plus::seed_from_u64(self.seed);
        self.time.start = Instant::now();
    }

    pub fn mino(&self) -> Mino {
        self.mino
    }

    pub fn board(&self) -> Board {
        self.board
    }

    pub fn block_iter<'a>(&'a self, y: u8) -> BlockIter<'a> {
        BlockIter::new(y, &self.board.0[y as usize].0, self.mino)
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
        let minos = [random_minos(rng), random_minos(rng)].concat();
        Self { held: None, minos }
    }

    fn gen_mino(&mut self, rng: &mut Xoshiro256Plus) -> Mino {
        let block = self.next_mino(rng);
        Mino {
            ori: Ori::Up,
            block,
            pos: (4, 0).into(),
            points: block.points(Ori::Up),
        }
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
            start: now,
            now,
            drop: Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self) -> Action {
        let now = Instant::now();
        let diff = now - self.now;
        self.now = now;
        self.drop += diff;

        if self.drop.as_millis() > 1_600 {
            let mut drop = 0;
            while self.drop.as_millis() > 1_600 {
                self.drop -= Duration::from_millis(1_600);
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
        use Block::*;
        use Ori::*;

        match self {
            I => match ori {
                Up => [(0, 1), (1, 1), (2, 1), (3, 1)],
                Left => [(1, 0), (1, 1), (1, 2), (1, 3)],
                Down => [(0, 2), (1, 2), (2, 2), (3, 2)],
                Right => [(2, 0), (2, 1), (2, 2), (2, 3)],
            },
            J => match ori {
                Up => [(0, 1), (1, 1), (2, 1), (0, 0)],
                Left => [(1, 0), (1, 1), (1, 2), (0, 2)],
                Down => [(0, 1), (1, 1), (2, 1), (2, 2)],
                Right => [(1, 0), (1, 1), (1, 2), (2, 0)],
            },
            L => match ori {
                Up => [(0, 1), (1, 1), (2, 1), (2, 0)],
                Left => [(1, 0), (1, 1), (1, 2), (0, 0)],
                Down => [(0, 1), (1, 1), (2, 1), (0, 2)],
                Right => [(1, 0), (1, 1), (1, 2), (2, 2)],
            },
            O => match ori {
                Up => [(1, 0), (1, 1), (2, 0), (2, 1)],
                Left => [(1, 0), (1, 1), (2, 0), (2, 1)],
                Down => [(1, 0), (1, 1), (2, 0), (2, 1)],
                Right => [(1, 0), (1, 1), (2, 0), (2, 1)],
            },
            S => match ori {
                Up => [(1, 0), (2, 0), (0, 1), (1, 1)],
                Left => [(0, 0), (0, 1), (1, 1), (1, 2)],
                Down => [(1, 1), (2, 1), (0, 2), (1, 2)],
                Right => [(1, 0), (1, 1), (2, 1), (2, 2)],
            },
            T => match ori {
                Up => [(1, 0), (0, 1), (1, 1), (2, 1)],
                Left => [(1, 0), (0, 1), (1, 1), (1, 2)],
                Down => [(0, 1), (1, 1), (2, 1), (1, 2)],
                Right => [(1, 0), (1, 1), (2, 1), (1, 2)],
            },
            Z => match ori {
                Up => [(0, 0), (1, 0), (1, 1), (2, 1)],
                Left => [(1, 0), (0, 1), (1, 1), (0, 2)],
                Down => [(0, 1), (1, 1), (1, 2), (2, 2)],
                Right => [(2, 0), (1, 1), (2, 1), (1, 2)],
            },
        }
        .map(Point::from)
    }
}

impl Mino {
    fn real_points(self) -> [Point; 4] {
        self.points.map(|p| self.pos + p)
    }
}

impl<'a> BlockIter<'a> {
    fn new(y: u8, block: &'a [Option<Block>], mino: Mino) -> Self {
        Self {
            mino_points: mino.real_points(),
            mino_block: mino.block,
            block,
            index: 0,
            y,
        }
    }
    fn block_or_mino(&self) -> Option<Block> {
        if self.mino_points.contains(&(self.index, self.y).into()) {
            return Some(self.mino_block);
        }
        self.block[self.index as usize]
    }
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = Option<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index as usize >= self.block.len() {
            return None;
        }
        let n = self.block_or_mino();
        self.index += 1;
        Some(n)
    }
}
