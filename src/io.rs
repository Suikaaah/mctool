use crate::grid::Grid;
use image::ImageReader;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    self as kam, INPUT, INPUT_0, KEYBDINPUT, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging as wam;

pub const INV_WIDTH: u32 = 486;
pub const INV_HEIGHT: u32 = 228;
pub const ITEM_WIDTH: u32 = 78;
pub const ITEM_HEIGHT: u32 = 78;
pub const FILENAME_THUMBNAIL: &str = "thumbnail.png";
pub const FILENAME_ITEM: &str = "item.png";
pub const FILENAME_CLICKS: &str = "clicks.json";

pub enum MouseButton {
    Left,
    Right,
}

pub fn is_down(vkey: VIRTUAL_KEY) -> bool {
    unsafe { kam::GetAsyncKeyState(vkey.0 as i32) }.cast_unsigned() >> 15 != 0
}

pub fn get_cursor() -> (i32, i32) {
    let mut point = windows::Win32::Foundation::POINT::default();

    unsafe { wam::GetCursorPos(&mut point) }.expect("failed to get cursor position");

    (point.x, point.y)
}

pub fn set_cursor(x: i32, y: i32) {
    unsafe { wam::SetCursorPos(x, y) }.expect("failed to set cursor position")
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

pub fn send_mouse(button: MouseButton) {
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

    let json = File::create_new(dir.join(FILENAME_CLICKS)).expect("failed to create json");

    serde_json::to_writer(json, clicks).expect("failed to write json");

    crop_latest_png(
        screenshots,
        dir.join(FILENAME_THUMBNAIL),
        dir.join(FILENAME_ITEM),
    );
}

pub fn load_clicks(path: impl AsRef<Path>) -> Box<[Grid]> {
    let file = std::fs::File::open(path).expect("failed to open file");
    serde_json::from_reader(file).expect("failed to parse json")
}

pub fn recipes(recipes: impl AsRef<Path>) -> Box<[PathBuf]> {
    std::fs::read_dir(recipes)
        .expect("failed to read directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect()
}

fn crop_latest_png<P, Q, R>(search_in: P, dst_inv: Q, dst_item: R)
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    let (path_inv, path_item) = get_latest_pngs(search_in).expect("directory or image not found");

    let mut image_inv = ImageReader::open(path_inv)
        .expect("failed to open image")
        .decode()
        .expect("failed to decode image");

    let mut image_item = ImageReader::open(path_item)
        .expect("failed to open image")
        .decode()
        .expect("failed to decode image");

    image_inv
        .crop(717, 540, INV_WIDTH, INV_HEIGHT)
        .save(dst_inv)
        .expect("failed to save image");

    image_item
        .crop(1053, 381, ITEM_WIDTH, ITEM_HEIGHT)
        .save(dst_item)
        .expect("failed to save image");
}

// (second latest, most latest)
fn get_latest_pngs(path: impl AsRef<Path>) -> Option<(PathBuf, PathBuf)> {
    struct Entry {
        modified: SystemTime,
        path: PathBuf,
    }

    let mut entries: Box<[Entry]> = std::fs::read_dir(path)
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
        .collect();

    entries.sort_unstable_by_key(|entry| entry.modified);
    entries
        .last_chunk::<2>()
        .map(|[a, b]| (a.path.clone(), b.path.clone()))
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        kam::SendInput(inputs, size_of::<INPUT>() as i32);
    }
}
