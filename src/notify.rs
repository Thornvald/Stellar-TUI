use std::path::{Path, PathBuf};

pub fn on_build_success() {
    play_sound("completed.wav");
    flash_taskbar();
}

pub fn on_build_failed() {
    play_sound("fail.wav");
    flash_taskbar();
}

fn sound_candidates(file_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(file_name));
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join(file_name));
                if let Some(grand) = parent.parent() {
                    candidates.push(grand.join(file_name));
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join(file_name));
    }

    candidates
}

fn play_sound(file_name: &str) {
    let mut played = false;
    for candidate in sound_candidates(file_name) {
        if candidate.exists() && play_sound_path(&candidate) {
            played = true;
            break;
        }
    }

    if !played {
        fallback_beep(file_name);
    }
}

#[cfg(windows)]
fn play_sound_path(path: &Path) -> bool {
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

    let mut wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().collect();
    wide.push(0);

    unsafe {
        PlaySoundW(
            wide.as_ptr(),
            std::ptr::null_mut(),
            SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
        ) != 0
    }
}

#[cfg(not(windows))]
fn play_sound_path(_path: &Path) -> bool {
    false
}

#[cfg(windows)]
fn fallback_beep(file_name: &str) {
    let _ = file_name;
    eprint!("\x07");
}

#[cfg(not(windows))]
fn fallback_beep(_file_name: &str) {}

#[cfg(windows)]
fn flash_taskbar() {
    use std::mem::size_of;
    use windows_sys::Win32::System::Console::GetConsoleWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FlashWindowEx, GetForegroundWindow, FLASHWINFO, FLASHW_CAPTION, FLASHW_TIMERNOFG,
        FLASHW_TRAY,
    };

    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.is_null() {
            return;
        }

        let foreground = GetForegroundWindow();
        if foreground == hwnd {
            return;
        }

        let mut info = FLASHWINFO {
            cbSize: size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_TRAY | FLASHW_CAPTION | FLASHW_TIMERNOFG,
            uCount: 5,
            dwTimeout: 0,
        };
        let _ = FlashWindowEx(&mut info);
    }
}

#[cfg(not(windows))]
fn flash_taskbar() {}
