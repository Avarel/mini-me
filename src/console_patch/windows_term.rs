
use std::os::windows::io::AsRawHandle;
use std::{io, mem};
use console::Term;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::MAX_PATH;
use winapi::um::fileapi::FILE_NAME_INFO;
use winapi::um::minwinbase::FileNameInfo;
use winapi::um::winbase::GetFileInformationByHandleEx;
use winapi::um::wincon::{
    GetConsoleScreenBufferInfo, SetConsoleCursorPosition, CONSOLE_SCREEN_BUFFER_INFO, COORD,
};

use winapi::um::winnt::{HANDLE, WCHAR};

use crate::console_patch::common_term;

pub fn move_cursor_right(out: &Term, n: usize) -> io::Result<()> {
    if msys_tty_on(out) {
        return common_term::move_cursor_right(out, n);
    }
    if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: csbi.dwCursorPosition.X + n as i16,
                    Y: csbi.dwCursorPosition.Y,
                },
            );
        }
    }
    Ok(())
}

pub fn move_cursor_left(out: &Term, n: usize) -> io::Result<()> {
    if msys_tty_on(out) {
        return common_term::move_cursor_left(out, n);
    }
    if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: csbi.dwCursorPosition.X - n as i16,
                    Y: csbi.dwCursorPosition.Y,
                },
            );
        }
    }
    Ok(())
}

pub fn move_cursor_up(out: &Term, n: usize) -> io::Result<()> {
    if msys_tty_on(out) {
        return common_term::move_cursor_up(out, n);
    }
    if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: csbi.dwCursorPosition.X,
                    Y: csbi.dwCursorPosition.Y - n as i16,
                },
            );
        }
    }
    Ok(())
}

pub fn move_cursor_down(out: &Term, n: usize) -> io::Result<()> {
    if msys_tty_on(out) {
        return common_term::move_cursor_down(out, n);
    }
    if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: csbi.dwCursorPosition.X,
                    Y: csbi.dwCursorPosition.Y + n as i16,
                },
            );
        }
    }
    Ok(())
}

fn get_console_screen_buffer_info(hand: HANDLE) -> Option<(HANDLE, CONSOLE_SCREEN_BUFFER_INFO)> {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
    match unsafe { GetConsoleScreenBufferInfo(hand, &mut csbi) } {
        0 => None,
        _ => Some((hand, csbi)),
    }
}

/// Returns true if there is an MSYS tty on the given handle.
pub fn msys_tty_on(term: &Term) -> bool {
    let handle = term.as_raw_handle();
    unsafe {
        let size = mem::size_of::<FILE_NAME_INFO>();
        let mut name_info_bytes = vec![0u8; size + MAX_PATH * mem::size_of::<WCHAR>()];
        let res = GetFileInformationByHandleEx(
            handle as *mut _,
            FileNameInfo,
            &mut *name_info_bytes as *mut _ as *mut c_void,
            name_info_bytes.len() as u32,
        );
        if res == 0 {
            return false;
        }
        let name_info: &FILE_NAME_INFO = &*(name_info_bytes.as_ptr() as *const FILE_NAME_INFO);
        let s = std::slice::from_raw_parts(
            name_info.FileName.as_ptr(),
            name_info.FileNameLength as usize / 2,
        );
        let name = String::from_utf16_lossy(s);
        // This checks whether 'pty' exists in the file name, which indicates that
        // a pseudo-terminal is attached. To mitigate against false positives
        // (e.g., an actual file name that contains 'pty'), we also require that
        // either the strings 'msys-' or 'cygwin-' are in the file name as well.)
        let is_msys = name.contains("msys-") || name.contains("cygwin-");
        let is_pty = name.contains("-pty");
        is_msys && is_pty
    }
}

pub fn as_handle(term: &Term) -> HANDLE {
    // convert between winapi::um::winnt::HANDLE and std::os::windows::raw::HANDLE
    // which are both c_void. would be nice to find a better way to do this
    unsafe { std::mem::transmute(term.as_raw_handle()) }
}