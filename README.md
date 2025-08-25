# mctool
A set of utilities for old Minecraft (e.g. 1.8.9), which requires no window focus and is easily accessible with your left hand.

![thumbnail](https://github.com/Suikaaah/mctool/blob/main/thumbnail.png)

## Features
- **Craft Recorder/Player**
- **Auto Clicker**
  - **Mouse buttons**
  - **Space bar**: Helps you travel at the maximum speed on ice rails
  - **Additional Right Click (ARC)**: Performs a right click following yours, making it easier to fill wide areas
  - **Trade**: Trades items without going back and forth with your mouse
- **Lock**: Temporarily disables inputs

## Default Key Bindings
You can change these in `Keys` at `src/state/key.rs`.
| Action | Key |
|---|---|
| Toggle Auto Clicker (Left) | Z |
| Toggle Auto Clicker (Right) | X |
| Toggle Auto Clicker (Space) | C |
| Start/Stop Recording | B |
| Craft | G |
| View Previous Recipe | Mouse "Forward" Button |
| View Next Recipe | Mouse "Back" Button |
| Toggle Additional Right Click (ARC) | Tab |
| Begin Trade | R |
| End Trade | Left Shift |
| Abort | Backtick |
| Toggle Lock | Left Control + Scroll Wheel Button |
| Temporarily Disable ARC | Left Control |
| Confirm (Save Recipe) | Return |
| View Previous Recipe (based on the first character) | Left Control + \<View Previous Recipe\> |
| View Next Recipe (based on the first character) | Left Control + \<View Next Recipe\> |

## Note
- A resolution of 1920 * 1080 and "Large" GUIs are expected
- Window needs to be focused upon recipe save in order to prevent the game from making unwanted reactions
- The latest screenshot and the second latest will be used to make thumbnails of your inventory and the
  resulting item, respectively

## Usage
- Set `SCREENSHOTS` and `RECIPES` in `src/state.rs` for your environment
- `cargo run --release`

## Build/Run
### Windows
- Download the development releases [SDL2](https://github.com/libsdl-org/SDL/releases/tag/release-2.32.8) | [SDL2_ttf](https://github.com/libsdl-org/SDL_ttf/releases/tag/release-2.24.0) | [SDL2_image](https://github.com/libsdl-org/SDL_image/releases/tag/release-2.8.8)
- Copy `*.lib` to the library directory of your Rust compiler
- Copy `*.dll` to `System32` or to the directory where your executable will be

### Linux
- Not available due to platform-specific IO implementations
