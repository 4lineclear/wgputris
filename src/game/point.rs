use std::ops::{Add, Mul};

/// A point on the board
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

/// A point on the board
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IPoint {
    pub x: i8,
    pub y: i8,
}
impl IPoint {
    fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

impl Point {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub fn xy(self) -> (u8, u8) {
        (self.x, self.y)
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y),
        }
    }
}

impl Add<IPoint> for Point {
    type Output = Self;

    fn add(self, rhs: IPoint) -> Self::Output {
        Self {
            x: self.x.saturating_add_signed(rhs.x),
            y: self.y.saturating_add_signed(rhs.y),
        }
    }
}

impl Mul<u32> for Point {
    type Output = (u32, u32);

    fn mul(self, v: u32) -> Self::Output {
        self * (v, v)
    }
}

impl Mul<(u32, u32)> for Point {
    type Output = (u32, u32);

    fn mul(self, (x, y): (u32, u32)) -> Self::Output {
        (x * self.x as u32, y * self.y as u32)
    }
}

impl From<(u8, u8)> for Point {
    fn from((x, y): (u8, u8)) -> Self {
        Self::new(x, y)
    }
}

impl From<(i8, i8)> for IPoint {
    fn from((x, y): (i8, i8)) -> Self {
        Self::new(x, y)
    }
}
