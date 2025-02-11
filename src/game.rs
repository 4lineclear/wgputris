/// A point on the board
#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub(crate) x: u8,
    pub(crate) y: u8,
}

impl Point {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

impl From<(u8, u8)> for Point {
    fn from((x, y): (u8, u8)) -> Self {
        Self::new(x, y)
    }
}

#[derive(Debug, Default)]
pub struct Game {
    board: Board,
}

impl Game {
    pub fn board(&self) -> Board {
        self.board
    }
}

/// The main board
///
/// Higher `y` is lower on the board
#[derive(Debug, Default, Clone, Copy)]
pub struct Board([Line; BOARD_HEIGHT as usize]);

pub const TOTAL_BLOCKS: u8 = BOARD_HEIGHT * BOARD_WIDTH;
pub const BOARD_HEIGHT: u8 = 24;
pub const BOARD_WIDTH: u8 = 10;

impl Board {
    /// Returns the visible lines
    pub fn visible(&self) -> &[Line] {
        &self.0[4..]
    }
    pub fn origin(&self) -> Point {
        Point::new(0, 23)
    }
    pub fn line(&self, y: impl Into<usize>) -> Line {
        self.0[y.into()]
    }
    pub fn block(&self, point: impl Into<Point>) -> Option<Block> {
        let Point { x, y } = point.into();
        self.line(y).block(x)
    }
}

/// A single line
#[derive(Debug, Default, Clone, Copy)]
pub struct Line([Option<Block>; 20]);

impl Line {
    pub fn block(&self, x: impl Into<usize>) -> Option<Block> {
        self.0[x.into()]
    }
}

/// A single block
#[derive(Debug, Clone, Copy)]
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
