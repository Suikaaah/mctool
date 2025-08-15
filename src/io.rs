use crate::grid::Grid;
use image::ImageReader;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
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

pub fn get_cursor() -> (i32, i32) {
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

    send_inputs(&[input]);
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

    send_inputs(&[input]);
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

    send_inputs(&[input]);
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

    send_inputs(&[input]);
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

    send_inputs(&[down, up]);
}

pub fn save_clicks<P, Q>(screenshots: P, recipes: Q, clicks: &[Grid])
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let timestamp = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("failed to obtain duration")
        .as_secs()
        .to_string();

    let dir = recipes.as_ref().join(timestamp);

    std::fs::create_dir(&dir).expect("failed to create directory");

    let json = File::create_new(dir.join("clicks.json")).expect("failed to create json");

    serde_json::to_writer(json, clicks).expect("failed to write json");

    crop_latest_png(screenshots, dir.join("thumbnail.png"));
}

pub fn _recipes(recipes: impl AsRef<Path>) -> Box<[PathBuf]> {
    std::fs::read_dir(recipes)
        .expect("failed to read directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect()
}

fn crop_latest_png(search_in: impl AsRef<Path>, dst: impl AsRef<Path>) {
    match get_latest_png(search_in) {
        Some(path) => {
            let mut image = ImageReader::open(path)
                .expect("failed to open image")
                .decode()
                .expect("failed to decode image");

            image
                .crop(717, 540, 486, 228)
                .save(dst)
                .expect("failed to save image");
        }
        None => println!("directory or image not found"),
    }
}

fn get_latest_png(path: impl AsRef<Path>) -> Option<PathBuf> {
    struct Entry {
        modified: SystemTime,
        path: PathBuf,
    }

    std::fs::read_dir(path)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry.metadata().ok().and_then(|metadata| {
                metadata.modified().ok().map(|modified| Entry {
                    modified,
                    path: entry.path(),
                })
            })
        })
        .filter(|entry| {
            entry
                .path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .max_by_key(|entry| entry.modified)
        .map(|entry| entry.path)
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        kam::SendInput(inputs, size_of::<INPUT>() as i32);
    }
}
