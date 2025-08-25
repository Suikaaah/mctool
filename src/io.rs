use crate::grid::Grid;
use crate::map_err_anyhow::MapErrAnyhow;
use anyhow::{Result, anyhow, bail};
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

pub fn get_cursor() -> Result<(i32, i32)> {
    let mut point = windows::Win32::Foundation::POINT::default();

    unsafe {
        wam::GetCursorPos(&mut point)?;
    }

    Ok((point.x, point.y))
}

pub fn set_cursor(x: i32, y: i32) -> Result<()> {
    unsafe { wam::SetCursorPos(x, y) }.map_err_anyhow()
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

pub fn save_clicks<P1, P2>(screenshots: P1, recipes: P2, clicks: &[Grid], name: &str) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // special message for an empty name (even without this, create_dir would return an error)
    if name.is_empty() {
        bail!("cannot save with empty name");
    }

    let dir = recipes.as_ref().join(name);

    std::fs::create_dir(&dir)?;

    let result = File::create_new(dir.join(FILENAME_CLICKS))
        .map_err_anyhow()
        .and_then(|json| {
            crop_latest_png(
                screenshots,
                dir.join(FILENAME_THUMBNAIL),
                dir.join(FILENAME_ITEM),
            )?;

            serde_json::to_writer(json, clicks).map_err_anyhow()
        });

    if result.is_err() {
        std::fs::remove_dir_all(dir)?;
    }

    result
}

pub fn load_clicks(path: impl AsRef<Path>) -> Result<Box<[Grid]>> {
    let file = File::open(path)?;
    serde_json::from_reader(file).map_err_anyhow()
}

pub fn recipes(recipes: impl AsRef<Path>) -> Result<Box<[PathBuf]>> {
    let mut boxed: Box<[PathBuf]> = std::fs::read_dir(recipes)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();

    boxed.sort_unstable();

    Ok(boxed)
}

pub fn message_box<V1, V2>(msg: V1, title: V2) -> Result<()>
where
    V1: Into<Vec<u8>>,
    V2: Into<Vec<u8>>,
{
    use std::ffi::CString;
    use windows::core::PCSTR;

    let msg = CString::new(msg)?;
    let title = CString::new(title)?;

    unsafe {
        wam::MessageBoxA(
            None,
            PCSTR::from_raw(msg.as_bytes().as_ptr()),
            PCSTR::from_raw(title.as_bytes().as_ptr()),
            wam::MB_ICONINFORMATION,
        );
    }

    Ok(())
}

fn crop_latest_png<P1, P2, P3>(search_in: P1, dst_inv: P2, dst_item: P3) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    P3: AsRef<Path>,
{
    use image::ImageReader;

    let (path_inv, path_item) = get_latest_pngs(search_in)?;

    let mut image_inv = ImageReader::open(path_inv)?.decode()?;
    let mut image_item = ImageReader::open(path_item)?.decode()?;

    image_inv
        .crop(717, 540, INV_WIDTH, INV_HEIGHT)
        .save(dst_inv)?;

    image_item
        .crop(1053, 381, ITEM_WIDTH, ITEM_HEIGHT)
        .save(dst_item)
        .map_err_anyhow()
}

// (second latest, most latest)
fn get_latest_pngs(path: impl AsRef<Path>) -> Result<(PathBuf, PathBuf)> {
    struct Entry {
        modified: SystemTime,
        path: PathBuf,
    }

    let mut entries: Box<[Entry]> = std::fs::read_dir(path)?
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
        .ok_or_else(|| anyhow!("two screenshots are required"))
}

fn send_inputs(inputs: &[INPUT]) {
    unsafe {
        kam::SendInput(inputs, size_of::<INPUT>() as i32);
    }
}
