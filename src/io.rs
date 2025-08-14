use windows::Win32::UI::Input::KeyboardAndMouse::{
    self as kam, INPUT, INPUT_0, KEYBDINPUT, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging as wam;

type WinResult<T> = windows::core::Result<T>;

pub enum MouseButton {
    Left,
    Right,
}

pub fn is_down(vkey: VIRTUAL_KEY) -> bool {
    unsafe { kam::GetAsyncKeyState(vkey.0 as i32) }.cast_unsigned() >> 15 != 0
}

pub fn _get_cursor() -> (i32, i32) {
    let mut point = windows::Win32::Foundation::POINT::default();

    if let WinResult::Err(e) = unsafe { wam::GetCursorPos(&mut point) } {
        println!("{e}");
    }

    (point.x, point.y)
}

pub fn set_cursor(x: i32, y: i32) {
    if let WinResult::Err(e) = unsafe { wam::SetCursorPos(x, y) } {
        println!("{e}");
    }
}

pub fn send_key_down(vkey: VIRTUAL_KEY) {
    let input = INPUT {
        r#type: kam::INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vkey,
                ..Default::default()
            },
        },
    };

    unsafe {
        kam::SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn send_key_up(vkey: VIRTUAL_KEY) {
    let input = INPUT {
        r#type: kam::INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vkey,
                dwFlags: kam::KEYEVENTF_KEYUP,
                ..Default::default()
            },
        },
    };

    unsafe {
        kam::SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn send_mouse_down(button: MouseButton) {
    let flag_down = match button {
        MouseButton::Left => kam::MOUSEEVENTF_LEFTDOWN,
        MouseButton::Right => kam::MOUSEEVENTF_RIGHTDOWN,
    };

    let input = INPUT {
        r#type: kam::INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: flag_down,
                ..Default::default()
            },
        },
    };

    unsafe {
        kam::SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn send_mouse_up(button: MouseButton) {
    let flag_up = match button {
        MouseButton::Left => kam::MOUSEEVENTF_LEFTUP,
        MouseButton::Right => kam::MOUSEEVENTF_RIGHTUP,
    };

    let input = INPUT {
        r#type: kam::INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: flag_up,
                ..Default::default()
            },
        },
    };

    unsafe {
        kam::SendInput(&[input], size_of::<INPUT>() as i32);
    }
}

pub fn _send_mouse(button: MouseButton) {
    let (flag_down, flag_up) = match button {
        MouseButton::Left => (kam::MOUSEEVENTF_LEFTDOWN, kam::MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (kam::MOUSEEVENTF_RIGHTDOWN, kam::MOUSEEVENTF_RIGHTUP),
    };

    let down = INPUT {
        r#type: kam::INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: flag_down,
                ..Default::default()
            },
        },
    };

    let up = INPUT {
        r#type: kam::INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: flag_up,
                ..Default::default()
            },
        },
    };

    unsafe {
        kam::SendInput(&[down, up], size_of::<INPUT>() as i32);
    }
}
