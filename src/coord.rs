#[derive(Clone, Copy)]
pub struct Coord(pub i32, pub i32);

impl Coord {
    pub const fn add(&self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }

    pub const fn sub(&self, rhs: Self) -> Self {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }

    pub const fn emul(&self, x: i32, y: i32) -> Self {
        Self(self.0 * x, self.1 * y)
    }

    pub const fn mul(&self, rhs: i32) -> Self {
        self.emul(rhs, rhs)
    }

    pub const fn ediv(&self, rhs: Self) -> Self {
        Self(self.0 / rhs.0, self.1 / rhs.1)
    }

    pub const fn div(&self, rhs: i32) -> Self {
        self.ediv(Self(rhs, rhs))
    }

    pub const fn is_contained(&self, top_left: Self, bottom_right: Self) -> bool {
        top_left.0 <= self.0
            && self.0 < bottom_right.0
            && top_left.1 <= self.1
            && self.1 < bottom_right.1
    }
}

impl From<(i32, i32)> for Coord {
    fn from((x, y): (i32, i32)) -> Self {
        Self(x, y)
    }
}

impl From<Coord> for (i32, i32) {
    fn from(value: Coord) -> Self {
        (value.0, value.1)
    }
}
