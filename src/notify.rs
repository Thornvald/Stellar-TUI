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
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::System::Console::GetConsoleWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FlashWindowEx, GetAncestor, GetForegroundWindow, GetWindow, FLASHWINFO, FLASHW_ALL,
        FLASHW_TIMERNOFG, GA_ROOTOWNER, GW_OWNER,
    };

    unsafe {
        let mut handles: Vec<HWND> = Vec::new();

        // 1) Console window and its ancestors/owner.
        let console = GetConsoleWindow();
        if !console.is_null() {
            handles.push(console);
            let root = GetAncestor(console, GA_ROOTOWNER);
            if !root.is_null() && root != console {
                handles.push(root);
            }
            let owner = GetWindow(console, GW_OWNER);
            if !owner.is_null() && owner != console {
                handles.push(owner);
            }
        }

        // 2) All visible windows owned by our process.
        let pid = std::process::id();
        let mut found = collect_windows_for_pid(pid);
        for h in found.drain(..) {
            if !handles.contains(&h) {
                handles.push(h);
            }
        }

        // 3) Also flash parent process windows (terminal host like Windows Terminal).
        if let Some(parent_pid) = get_parent_pid() {
            let mut parent_found = collect_windows_for_pid(parent_pid);
            for h in parent_found.drain(..) {
                if !handles.contains(&h) {
                    handles.push(h);
                }
            }
        }

        let foreground = GetForegroundWindow();

        // Flash every candidate window except the currently focused one.
        for hwnd in handles {
            if hwnd == foreground {
                continue;
            }
            let mut info = FLASHWINFO {
                cbSize: size_of::<FLASHWINFO>() as u32,
                hwnd,
                dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
                uCount: 5,
                dwTimeout: 0,
            };
            let _ = FlashWindowEx(&mut info);
        }
    }
}

#[cfg(windows)]
fn collect_windows_for_pid(target_pid: u32) -> Vec<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    // HWND is *mut c_void which is not Send, so store as usize for the static Mutex.
    static RESULTS: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let pid = lparam as u32;
        let mut window_pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, &mut window_pid);
        if window_pid == pid && IsWindowVisible(hwnd) != 0 {
            if let Ok(mut r) = RESULTS.lock() {
                r.push(hwnd as usize);
            }
        }
        1
    }

    if let Ok(mut r) = RESULTS.lock() {
        r.clear();
    }
    unsafe {
        let _ = EnumWindows(Some(enum_cb), target_pid as LPARAM);
    }
    RESULTS
        .lock()
        .map(|r| r.iter().map(|&h| h as HWND).collect())
        .unwrap_or_default()
}

#[cfg(windows)]
fn get_parent_pid() -> Option<u32> {
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap.is_null() {
            return None;
        }

        let my_pid = std::process::id();
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snap, &mut entry) != 0 {
            loop {
                if entry.th32ProcessID == my_pid {
                    let _ = windows_sys::Win32::Foundation::CloseHandle(snap);
                    return Some(entry.th32ParentProcessID);
                }
                if Process32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }

        let _ = windows_sys::Win32::Foundation::CloseHandle(snap);
        None
    }
}

#[cfg(not(windows))]
fn flash_taskbar() {}
