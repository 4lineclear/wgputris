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

impl From<IPoint> for Point {
    fn from(value: IPoint) -> Self {
        Point::new(value.x.unsigned_abs(), value.y.unsigned_abs())
    }
}

impl From<Point> for IPoint {
    fn from(value: Point) -> Self {
        IPoint::new((value.x as i8).abs(), (value.y as i8).abs())
    }
}

impl IPoint {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

impl Point {
    pub const fn new(x: u8, y: u8) -> Self {
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

impl Add for IPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x.saturating_add(rhs.x),
            y: self.y.saturating_add(rhs.y),
        }
    }
}

impl Add<IPoint> for Point {
    type Output = Option<IPoint>;

    fn add(self, rhs: IPoint) -> Self::Output {
        Some(IPoint {
            x: (self.x as i8).checked_add(rhs.x)?,
            y: (self.y as i8).checked_add(rhs.y)?,
        })
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

pub fn maybe_cast_points(value: [Option<IPoint>; 4]) -> Option<[Point; 4]> {
    let mut points = [Point::default(); 4];
    for i in 0..4 {
        let p = value[i]?;
        if p.x < 0 || p.y < 0 {
            return None;
        }
        points[i] = Point::new(p.x as u8, p.y as u8);
    }
    Some(points)
}

pub fn cast_points(value: [IPoint; 4]) -> Option<[Point; 4]> {
    let mut points = [Point::default(); 4];
    for i in 0..4 {
        let p = value[i];
        points[i] = Point::new(u8::try_from(p.x).ok()?, u8::try_from(p.y).ok()?);
    }
    Some(points)
}
