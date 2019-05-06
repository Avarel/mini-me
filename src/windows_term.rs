use winapi::um::wincon::{
    FillConsoleOutputCharacterA, GetConsoleScreenBufferInfo, SetConsoleCursorPosition,
    CONSOLE_SCREEN_BUFFER_INFO, COORD, INPUT_RECORD, KEY_EVENT, KEY_EVENT_RECORD,
};

use winapi::um::winnt::{CHAR, HANDLE, INT, WCHAR};

fn get_console_screen_buffer_info(hand: HANDLE) -> Option<(HANDLE, CONSOLE_SCREEN_BUFFER_INFO)> {
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = unsafe { mem::zeroed() };
    match unsafe { GetConsoleScreenBufferInfo(hand, &mut csbi) } {
        0 => None,
        _ => Some((hand, csbi)),
    }
}

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
                    Y: 0,
                },
            );
        }
    }
    Ok(())
}

pub fn move_cursor_left(out: &Term, n: usize) -> io::Result<()> {
    if msys_tty_on(out) {
        return common_term::move_cursor_right(out, n);
    }
    if let Some((hand, csbi)) = get_console_screen_buffer_info(as_handle(out)) {
        unsafe {
            SetConsoleCursorPosition(
                hand,
                COORD {
                    X: csbi.dwCursorPosition.X - n as i16,
                    Y: 0,
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