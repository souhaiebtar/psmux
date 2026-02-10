/// Enable virtual terminal processing on Windows Console Host.
/// This is required for ANSI color codes to work in conhost.exe (legacy console).
#[cfg(windows)]
pub fn enable_virtual_terminal_processing() {
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> *mut std::ffi::c_void;
        fn GetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: *mut std::ffi::c_void, dwMode: u32) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if !handle.is_null() {
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

#[cfg(not(windows))]
pub fn enable_virtual_terminal_processing() {
    // No-op on non-Windows platforms
}

/// Install a console control handler on Windows to prevent termination on client detach.
#[cfg(windows)]
pub fn install_console_ctrl_handler() {
    type HandlerRoutine = unsafe extern "system" fn(u32) -> i32;

    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCtrlHandler(handler: Option<HandlerRoutine>, add: i32) -> i32;
    }

    const CTRL_CLOSE_EVENT: u32 = 2;
    const CTRL_LOGOFF_EVENT: u32 = 5;
    const CTRL_SHUTDOWN_EVENT: u32 = 6;

    unsafe extern "system" fn handler(ctrl_type: u32) -> i32 {
        match ctrl_type {
            CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT => 1,
            _ => 0,
        }
    }

    unsafe {
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}

#[cfg(not(windows))]
pub fn install_console_ctrl_handler() {
    // No-op on non-Windows platforms
}

// ---------------------------------------------------------------------------
// Windows Console API mouse injection
// ---------------------------------------------------------------------------
// ConPTY does NOT translate VT mouse escape sequences (e.g. SGR \x1b[<0;10;5M)
// into MOUSE_EVENT INPUT_RECORDs. Writing them to the PTY master appears as
// garbage text in the child app.
//
// The solution: use WriteConsoleInput to inject native MOUSE_EVENT records
// directly into the child's console input buffer.
//
// Flow:
//   1. On first mouse event targeting a pane, lazily acquire the console handle:
//      FreeConsole() → AttachConsole(child_pid) → CreateFileW("CONIN$") → FreeConsole()
//   2. The handle remains valid after FreeConsole on modern Windows (real kernel handles).
//   3. Use WriteConsoleInputW(handle, MOUSE_EVENT record) for each mouse event.
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub mod mouse_inject {
    use std::ffi::c_void;

    const GENERIC_READ: u32  = 0x80000000;
    const GENERIC_WRITE: u32 = 0x40000000;
    const FILE_SHARE_READ: u32  = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;
    const OPEN_EXISTING: u32 = 3;
    const INVALID_HANDLE: isize = -1;

    const MOUSE_EVENT: u16 = 0x0002;
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;

    // dwButtonState flags
    pub const FROM_LEFT_1ST_BUTTON_PRESSED: u32 = 0x0001;
    pub const RIGHTMOST_BUTTON_PRESSED: u32     = 0x0002;
    pub const FROM_LEFT_2ND_BUTTON_PRESSED: u32 = 0x0004; // middle button

    // dwEventFlags
    pub const MOUSE_MOVED: u32       = 0x0001;
    pub const MOUSE_WHEELED: u32     = 0x0004;

    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static LAST_DRAG_INJECT: Mutex<Option<Instant>> = Mutex::new(None);
    const DRAG_THROTTLE: Duration = Duration::from_millis(16); // ~60fps

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct COORD {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct MOUSE_EVENT_RECORD {
        mouse_position: COORD,
        button_state: u32,
        control_key_state: u32,
        event_flags: u32,
    }

    #[repr(C)]
    struct INPUT_RECORD {
        event_type: u16,
        _padding: u16,
        event: MOUSE_EVENT_RECORD,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn FreeConsole() -> i32;
        fn AttachConsole(process_id: u32) -> i32;
        fn GetConsoleWindow() -> isize;
        fn CreateFileW(
            file_name: *const u16,
            desired_access: u32,
            share_mode: u32,
            security_attributes: *const c_void,
            creation_disposition: u32,
            flags_and_attributes: u32,
            template_file: *const c_void,
        ) -> isize;
        fn WriteConsoleInputW(
            console_input: isize,
            buffer: *const INPUT_RECORD,
            length: u32,
            events_written: *mut u32,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
        fn GetProcessId(process: isize) -> u32;
        fn GetLastError() -> u32;
    }

    #[inline]
    fn debug_log(_msg: &str) {
        // Debug logging disabled for performance.
        // To re-enable: write to $TEMP/psmux_mouse_debug.log
    }

    /// Extract the process ID from a portable_pty::Child trait object.
    ///
    /// SAFETY: On Windows with ConPTY (portable_pty 0.2), the concrete type behind
    /// `dyn Child` is `WinChild { proc: OwnedHandle }` where OwnedHandle wraps a
    /// single Windows HANDLE. We read the HANDLE and call GetProcessId.
    pub unsafe fn get_child_pid(child: &dyn portable_pty::Child) -> Option<u32> {
        let data_ptr = child as *const dyn portable_pty::Child as *const u8;
        let handle = *(data_ptr as *const isize);
        debug_log(&format!("get_child_pid: data_ptr={:p} handle=0x{:X}", data_ptr, handle));
        if handle == 0 || handle == -1 {
            debug_log("get_child_pid: INVALID handle");
            return None;
        }
        let pid = GetProcessId(handle);
        let err = GetLastError();
        debug_log(&format!("get_child_pid: GetProcessId(0x{:X}) => pid={} err={}", handle, pid, err));
        if pid == 0 { None } else { Some(pid) }
    }

    /// Inject a mouse event into a child process's console input buffer.
    ///
    /// Performs the full cycle: FreeConsole → AttachConsole(pid) → open CONIN$
    /// → WriteConsoleInputW → CloseHandle → FreeConsole.
    ///
    /// Console handles are pseudo-handles that are invalidated by FreeConsole,
    /// so we must do the entire cycle atomically for each event.
    ///
    /// `reattach`: if true, re-attaches to original console after injection
    /// (needed for app/standalone mode where crossterm uses the console).
    /// Server mode should pass false to avoid conhost cycling.
    pub fn send_mouse_event(
        child_pid: u32,
        col: i16,
        row: i16,
        button_state: u32,
        event_flags: u32,
        reattach: bool,
    ) -> bool {
        // Throttle drag events to ~60fps to avoid excessive console attach/detach cycling
        if event_flags & MOUSE_MOVED != 0 {
            if let Ok(mut guard) = LAST_DRAG_INJECT.lock() {
                if let Some(t) = *guard {
                    if t.elapsed() < DRAG_THROTTLE {
                        return false;
                    }
                }
                *guard = Some(Instant::now());
            }
        }

        unsafe {
            // Check if we currently own a console (app mode yes, server mode no after first call)
            let had_console = reattach && GetConsoleWindow() != 0;

            // Detach from current console (no-op if already detached)
            FreeConsole();

            // Attach to child's pseudo-console
            if AttachConsole(child_pid) == 0 {
                let err = GetLastError();
                debug_log(&format!("send_mouse_event: AttachConsole({}) FAILED err={}", child_pid, err));
                if had_console { AttachConsole(ATTACH_PARENT_PROCESS); }
                return false;
            }

            // Open the console input buffer
            let conin: [u16; 7] = [
                'C' as u16, 'O' as u16, 'N' as u16,
                'I' as u16, 'N' as u16, '$' as u16, 0,
            ];
            let handle = CreateFileW(
                conin.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null(),
                OPEN_EXISTING,
                0,
                std::ptr::null(),
            );

            if handle == INVALID_HANDLE || handle == 0 {
                let err = GetLastError();
                debug_log(&format!("send_mouse_event: CreateFileW(CONIN$) FAILED err={}", err));
                FreeConsole();
                if had_console { AttachConsole(ATTACH_PARENT_PROCESS); }
                return false;
            }

            // Write the mouse event
            let record = INPUT_RECORD {
                event_type: MOUSE_EVENT,
                _padding: 0,
                event: MOUSE_EVENT_RECORD {
                    mouse_position: COORD { x: col, y: row },
                    button_state,
                    control_key_state: 0,
                    event_flags,
                },
            };
            let mut written: u32 = 0;
            let result = WriteConsoleInputW(handle, &record, 1, &mut written);
            let write_err = GetLastError();

            debug_log(&format!("send_mouse_event: pid={} ({},{}) btn=0x{:X} flags=0x{:X} => ok={} written={} err={}",
                child_pid, col, row, button_state, event_flags, result, written, write_err));

            // Clean up: close handle, detach from child's console
            CloseHandle(handle);
            FreeConsole();
            // Only re-attach if we had our own console (app/standalone mode)
            // Server mode: leave detached to avoid conhost cycling
            if had_console {
                AttachConsole(ATTACH_PARENT_PROCESS);
            }

            result != 0
        }
    }
}

#[cfg(not(windows))]
pub mod mouse_inject {
    pub unsafe fn get_child_pid(_child: &dyn portable_pty::Child) -> Option<u32> { None }
    pub fn send_mouse_event(_pid: u32, _col: i16, _row: i16, _btn: u32, _flags: u32, _reattach: bool) -> bool { false }
}
