use windows::Win32::UI::Input::KeyboardAndMouse::{self as kam, VIRTUAL_KEY};

pub struct Key {
    vkeys: Box<[VIRTUAL_KEY]>,
    previous: bool,
    is_pressed: bool,
    is_released: bool,
}

impl Key {
    pub fn single(vkey: VIRTUAL_KEY) -> Self {
        Self {
            vkeys: Box::new([vkey]),
            previous: false,
            is_pressed: false,
            is_released: false,
        }
    }

    pub fn multiple(vkeys: impl Into<Box<[VIRTUAL_KEY]>>) -> Self {
        Self {
            vkeys: vkeys.into(),
            previous: false,
            is_pressed: false,
            is_released: false,
        }
    }

    pub fn update(&mut self, is_disabled: bool) {
        let is_down = !is_disabled && self.vkeys.iter().all(|vkey| crate::io::is_down(*vkey));

        self.is_pressed = is_down && !self.previous;
        self.is_released = !is_down && self.previous;
        self.previous = is_down;
    }

    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub const fn is_released(&self) -> bool {
        self.is_released
    }
}

pub struct Keys {
    pub left: Key,
    pub right: Key,
    pub space: Key,
    pub record: Key,
    pub play: Key,
    pub click: Key,
    pub right_click: Key,
    pub prev: Key,
    pub next: Key,
    pub double_click: Key,
    pub begin_trade: Key,
    pub end_trade: Key,
    pub abort: Key,
    pub lock: Key,
    pub cancel_dc: Key,
    pub confirm: Key,
    pub prev_skip: Key,
    pub next_skip: Key,
}

impl Keys {
    const LEFT: VIRTUAL_KEY = kam::VK_Z;
    const RIGHT: VIRTUAL_KEY = kam::VK_X;
    const SPACE: VIRTUAL_KEY = kam::VK_C;
    const RECORD: VIRTUAL_KEY = kam::VK_B;
    const PLAY: VIRTUAL_KEY = kam::VK_G;
    const CLICK: VIRTUAL_KEY = kam::VK_LBUTTON;
    const RIGHT_CLICK: VIRTUAL_KEY = kam::VK_RBUTTON;
    const PREV: VIRTUAL_KEY = kam::VK_XBUTTON2;
    const NEXT: VIRTUAL_KEY = kam::VK_XBUTTON1;
    const DOUBLE_CLICK: VIRTUAL_KEY = kam::VK_TAB;
    const BEGIN_TRADE: VIRTUAL_KEY = kam::VK_R;
    const END_TRADE: VIRTUAL_KEY = kam::VK_LSHIFT;
    const ABORT: VIRTUAL_KEY = kam::VK_OEM_3;
    const LOCK: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, kam::VK_MBUTTON];
    pub const CANCEL_DC: VIRTUAL_KEY = kam::VK_LCONTROL;
    const CONFIRM: VIRTUAL_KEY = kam::VK_RETURN;
    const PREV_SKIP: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, Self::PREV];
    const NEXT_SKIP: &[VIRTUAL_KEY] = &[kam::VK_LCONTROL, Self::NEXT];

    pub fn new() -> Self {
        Self {
            left: Key::single(Self::LEFT),
            right: Key::single(Self::RIGHT),
            space: Key::single(Self::SPACE),
            record: Key::single(Self::RECORD),
            play: Key::single(Self::PLAY),
            click: Key::single(Self::CLICK),
            right_click: Key::single(Self::RIGHT_CLICK),
            prev: Key::single(Self::PREV),
            next: Key::single(Self::NEXT),
            double_click: Key::single(Self::DOUBLE_CLICK),
            begin_trade: Key::single(Self::BEGIN_TRADE),
            end_trade: Key::single(Self::END_TRADE),
            abort: Key::single(Self::ABORT),
            lock: Key::multiple(Self::LOCK),
            cancel_dc: Key::single(Self::CANCEL_DC),
            confirm: Key::single(Self::CONFIRM),
            prev_skip: Key::multiple(Self::PREV_SKIP),
            next_skip: Key::multiple(Self::NEXT_SKIP),
        }
    }
}
