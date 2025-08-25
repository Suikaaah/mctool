use crate::grid::Grid;
use std::time::Instant;

pub enum Detail {
    Idle,
    Recording {
        clicks: Vec<Grid>,
    },
    Naming {
        clicks: Vec<Grid>,
        name: String,
        draw_required: bool,
    },
    Playing {
        clicks: Box<[(Grid, Cursor)]>,
        origin: Instant,
    },
    TradingFirst {
        state: TradeFirst,
        position: (i32, i32),
        origin: Instant,
    },
    TradingSecond {
        state: TradeSecond,
        position: (i32, i32),
        origin: Instant,
    },
}

pub enum Cursor {
    New,
    Moved,
    Clicked,
}

pub enum TradeFirst {
    InvClicked,
    MovedToLeft,
    LeftClicked,
    Waiting,
}

pub enum TradeSecond {
    RightClicked,
    MovedToLeft,
    LeftClicked,
}
