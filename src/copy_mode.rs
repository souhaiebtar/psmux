use std::io::{self, Write};
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{GlobalFree, HGLOBAL};
#[cfg(windows)]
use windows_sys::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

use crate::types::*;
use crate::tree::*;

pub fn enter_copy_mode(app: &mut AppState) { 
    app.mode = Mode::CopyMode; 
    app.copy_scroll_offset = 0;
}

#[cfg(windows)]
pub fn copy_to_system_clipboard(text: &str) {
    const CF_UNICODETEXT: u32 = 13;

    // Clipboard can be momentarily locked by other processes; retry briefly.
    for _ in 0..5 {
        let opened = unsafe { OpenClipboard(std::ptr::null_mut()) };
        if opened == 0 {
            thread::sleep(Duration::from_millis(2));
            continue;
        }

        let mut utf16: Vec<u16> = text.encode_utf16().collect();
        utf16.push(0); // null terminator required by CF_UNICODETEXT
        let size_bytes = utf16.len() * std::mem::size_of::<u16>();
        let mut hmem: HGLOBAL = std::ptr::null_mut();

        unsafe {
            if EmptyClipboard() != 0 {
                hmem = GlobalAlloc(GMEM_MOVEABLE, size_bytes);
                if !hmem.is_null() {
                    let dst = GlobalLock(hmem) as *mut u16;
                    if !dst.is_null() {
                        std::ptr::copy_nonoverlapping(utf16.as_ptr(), dst, utf16.len());
                        GlobalUnlock(hmem);
                        if !SetClipboardData(CF_UNICODETEXT, hmem).is_null() {
                            // Ownership transferred to the OS on success.
                            hmem = std::ptr::null_mut();
                        }
                    }
                }
            }

            if !hmem.is_null() {
                let _ = GlobalFree(hmem);
            }
            let _ = CloseClipboard();
        }
        break;
    }
}

#[cfg(not(windows))]
pub fn copy_to_system_clipboard(_text: &str) {}

/// Read text from the Windows system clipboard.
#[cfg(windows)]
pub fn read_from_system_clipboard() -> Option<String> {
    const CF_UNICODETEXT: u32 = 13;
    for _ in 0..5 {
        let opened = unsafe { OpenClipboard(std::ptr::null_mut()) };
        if opened == 0 {
            thread::sleep(Duration::from_millis(2));
            continue;
        }
        let result = unsafe {
            let hmem = GetClipboardData(CF_UNICODETEXT);
            if hmem.is_null() {
                let _ = CloseClipboard();
                return None;
            }
            let ptr = GlobalLock(hmem) as *const u16;
            if ptr.is_null() {
                let _ = CloseClipboard();
                return None;
            }
            // Find null terminator
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
                if len > 1_000_000 { break; } // safety limit
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16_lossy(slice);
            GlobalUnlock(hmem);
            let _ = CloseClipboard();
            Some(text)
        };
        return result;
    }
    None
}

#[cfg(not(windows))]
pub fn read_from_system_clipboard() -> Option<String> { None }

pub fn current_prompt_pos(app: &mut AppState) -> Option<(u16,u16)> {
    let win = &mut app.windows[app.active_idx];
    let p = active_pane_mut(&mut win.root, &win.active_path)?;
    let parser = p.term.lock().ok()?;
    let (r,c) = parser.screen().cursor_position();
    Some((r,c))
}

pub fn move_copy_cursor(app: &mut AppState, dx: i16, dy: i16) {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return };
    let parser = match p.term.lock() { Ok(g) => g, Err(_) => return };
    let (r,c) = parser.screen().cursor_position();
    let nr = (r as i16 + dy).max(0) as u16;
    let nc = (c as i16 + dx).max(0) as u16;
    app.copy_pos = Some((nr,nc));
}

pub fn scroll_copy_up(app: &mut AppState, lines: usize) {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return };
    let mut parser = match p.term.lock() { Ok(g) => g, Err(_) => return };
    let current = parser.screen().scrollback();
    let new_offset = current.saturating_add(lines);
    parser.screen_mut().set_scrollback(new_offset);
    app.copy_scroll_offset = parser.screen().scrollback();
}

pub fn scroll_copy_down(app: &mut AppState, lines: usize) {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return };
    let mut parser = match p.term.lock() { Ok(g) => g, Err(_) => return };
    let current = parser.screen().scrollback();
    let new_offset = current.saturating_sub(lines);
    parser.screen_mut().set_scrollback(new_offset);
    app.copy_scroll_offset = parser.screen().scrollback();
}

pub fn scroll_to_top(app: &mut AppState) {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return };
    let mut parser = match p.term.lock() { Ok(g) => g, Err(_) => return };
    parser.screen_mut().set_scrollback(usize::MAX);
    app.copy_scroll_offset = parser.screen().scrollback();
}

pub fn scroll_to_bottom(app: &mut AppState) {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return };
    let mut parser = match p.term.lock() { Ok(g) => g, Err(_) => return };
    parser.screen_mut().set_scrollback(0);
    app.copy_scroll_offset = 0;
}

pub fn yank_selection(app: &mut AppState) -> io::Result<()> {
    let (anchor, pos) = match (app.copy_anchor, app.copy_pos) { (Some(a), Some(p)) => (a,p), _ => return Ok(()) };
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return Ok(()) };
    let parser = match p.term.lock() { Ok(g) => g, Err(_) => return Ok(()) };
    let screen = parser.screen();
    let r0 = anchor.0.min(pos.0); let r1 = anchor.0.max(pos.0);
    let c0 = anchor.1.min(pos.1); let c1 = anchor.1.max(pos.1);
    let mut text = String::new();
    for r in r0..=r1 {
        for c in c0..=c1 {
            if let Some(cell) = screen.cell(r, c) { text.push_str(&cell.contents().to_string()); } else { text.push(' '); }
        }
        if r < r1 { text.push('\n'); }
    }
    app.paste_buffers.push(text.clone());
    copy_to_system_clipboard(&text);
    Ok(())
}

pub fn paste_latest(app: &mut AppState) -> io::Result<()> {
    if let Some(buf) = app.paste_buffers.last() {
        let win = &mut app.windows[app.active_idx];
        if let Some(p) = active_pane_mut(&mut win.root, &win.active_path) { let _ = write!(p.master, "{}", buf); }
    }
    Ok(())
}

pub fn capture_active_pane(app: &mut AppState) -> io::Result<()> {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return Ok(()) };
    let parser = match p.term.lock() { Ok(g) => g, Err(_) => return Ok(()) };
    let screen = parser.screen();
    let mut text = String::new();
    for r in 0..p.last_rows { for c in 0..p.last_cols { if let Some(cell) = screen.cell(r, c) { text.push_str(&cell.contents().to_string()); } else { text.push(' '); } } text.push('\n'); }
    app.paste_buffers.push(text);
    Ok(())
}

pub fn capture_active_pane_text(app: &mut AppState) -> io::Result<Option<String>> {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return Ok(None) };
    let parser = match p.term.lock() { Ok(g) => g, Err(_) => return Ok(None) };
    let screen = parser.screen();
    let mut text = String::new();
    for r in 0..p.last_rows { for c in 0..p.last_cols { if let Some(cell) = screen.cell(r, c) { text.push_str(&cell.contents().to_string()); } else { text.push(' '); } } text.push('\n'); }
    Ok(Some(text))
}

pub fn save_latest_buffer(app: &mut AppState, file: &str) -> io::Result<()> {
    if let Some(buf) = app.paste_buffers.last() { std::fs::write(file, buf)?; }
    Ok(())
}

pub fn capture_active_pane_range(app: &mut AppState, s: Option<u16>, e: Option<u16>) -> io::Result<Option<String>> {
    let win = &mut app.windows[app.active_idx];
    let p = match active_pane_mut(&mut win.root, &win.active_path) { Some(p) => p, None => return Ok(None) };
    let parser = match p.term.lock() { Ok(g) => g, Err(_) => return Ok(None) };
    let screen = parser.screen();
    let start = s.unwrap_or(0).min(p.last_rows.saturating_sub(1));
    let end = e.unwrap_or(p.last_rows.saturating_sub(1)).min(p.last_rows.saturating_sub(1));
    let mut text = String::new();
    for r in start..=end { for c in 0..p.last_cols { if let Some(cell) = screen.cell(r, c) { text.push_str(&cell.contents().to_string()); } else { text.push(' '); } } text.push('\n'); }
    Ok(Some(text))
}
