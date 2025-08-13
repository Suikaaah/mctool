mod coord;
use coord::Coord;

pub enum Grid {
    Table(u8, u8),
    Craft,
    Inv(u8, u8),
    Hotbar(u8),
}

impl Grid {
    const GRID_SIZE: Coord = Coord(54, 54);
    const ORIGIN_TABLE: Coord = Coord(783, 339);
    const ORIGIN_CRAFT: Coord = Coord(1065, 393);
    const ORIGIN_INV: Coord = Coord(717, 540);
    const ORIGIN_HOTBAR: Coord = Coord(717, 714);
    const END_TABLE: Coord = Self::ORIGIN_TABLE.add(Self::GRID_SIZE.mul(3));
    const END_CRAFT: Coord = Self::ORIGIN_CRAFT.add(Self::GRID_SIZE);
    const END_INV: Coord = Self::ORIGIN_INV.add(Self::GRID_SIZE.emul(9, 3));
    const END_HOTBAR: Coord = Self::ORIGIN_HOTBAR.add(Self::GRID_SIZE.emul(9, 1));

    pub fn set_cursor(&self) {
        let Coord(x, y) = self.into();
        crate::io::set_cursor(x, y);
    }
}

impl TryFrom<&Coord> for Grid {
    type Error = &'static str;

    fn try_from(value: &Coord) -> Result<Self, Self::Error> {
        let f = |origin| {
            let Coord(x, y) = value.sub(origin).ediv(Self::GRID_SIZE);
            (x as u8, y as u8)
        };

        if value.is_contained(Self::ORIGIN_TABLE, Self::END_TABLE) {
            let (x, y) = f(Self::ORIGIN_TABLE);
            Ok(Self::Table(x, y))
        } else if value.is_contained(Self::ORIGIN_CRAFT, Self::END_CRAFT) {
            Ok(Self::Craft)
        } else if value.is_contained(Self::ORIGIN_INV, Self::END_INV) {
            let (x, y) = f(Self::ORIGIN_INV);
            Ok(Self::Inv(x, y))
        } else if value.is_contained(Self::ORIGIN_HOTBAR, Self::END_HOTBAR) {
            let (x, _) = f(Self::ORIGIN_HOTBAR);
            Ok(Self::Hotbar(x))
        } else {
            Err("not on a grid")
        }
    }
}

impl From<&Grid> for Coord {
    fn from(value: &Grid) -> Self {
        let f = |origin: Coord, x, y| {
            origin
                .add(Grid::GRID_SIZE.emul(x as i32, y as i32))
                .add(Grid::GRID_SIZE.div(2))
        };

        match value {
            Grid::Table(x, y) => f(Grid::ORIGIN_TABLE, *x, *y),
            Grid::Craft => Grid::ORIGIN_CRAFT,
            Grid::Inv(x, y) => f(Grid::ORIGIN_INV, *x, *y),
            Grid::Hotbar(x) => f(Grid::ORIGIN_HOTBAR, *x, 0),
        }
    }
}
