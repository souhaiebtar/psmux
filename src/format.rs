use std::env;

use crate::types::*;
use crate::tree::*;
use crate::config::format_key_binding;

/// Expand tmux format strings in text for the active window.
/// Supports both #X shorthand and #{variable_name} syntax.
#[inline]
pub fn expand_format(fmt: &str, app: &AppState) -> String {
    expand_format_for_window(fmt, app, app.active_idx)
}

/// Expand tmux format strings for a specific window index.
pub fn expand_format_for_window(fmt: &str, app: &AppState, win_idx: usize) -> String {
    let mut result = String::with_capacity(fmt.len() * 2);
    let bytes = fmt.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'#' && i + 1 < len {
            if bytes[i + 1] == b'{' {
                // #{variable_name} syntax
                if let Some(close) = bytes[i+2..].iter().position(|&c| c == b'}') {
                    let var = &fmt[i+2..i+2+close];
                    result.push_str(&expand_format_var_for_window(var, app, win_idx));
                    i += close + 3; // skip #{ ... }
                    continue;
                }
            }
            // Shorthand #X format
            match bytes[i + 1] {
                b'S' => { result.push_str(&app.session_name); i += 2; continue; }
                b'I' => {
                    let n = win_idx + app.window_base_index;
                    result.push_str(&n.to_string());
                    i += 2; continue;
                }
                b'W' | b'T' => { result.push_str(&app.windows[win_idx].name); i += 2; continue; }
                b'P' => {
                    let win = &app.windows[win_idx];
                    let pid = get_active_pane_id(&win.root, &win.active_path).unwrap_or(0);
                    result.push_str(&pid.to_string());
                    i += 2; continue;
                }
                b'F' => {
                    if win_idx == app.active_idx { result.push('*'); }
                    else if win_idx == app.last_window_idx { result.push('-'); }
                    i += 2; continue;
                }
                b'H' | b'h' => {
                    result.push_str(&hostname_cached());
                    i += 2; continue;
                }
                b'D' => {
                    result.push_str(&chrono::Local::now().format("%Y-%m-%d").to_string());
                    i += 2; continue;
                }
                b'#' => { result.push('#'); i += 2; continue; }
                _ => {}
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    // Expand strftime %-sequences (e.g. %H:%M, %d-%b-%y) via chrono
    if result.contains('%') {
        result = chrono::Local::now().format(&result).to_string();
    }
    result
}

fn expand_format_var_for_window(var: &str, app: &AppState, win_idx: usize) -> String {
    let win = &app.windows[win_idx];
    match var {
        "session_name" => app.session_name.clone(),
        "session_attached" => if app.attached_clients > 0 { "1".into() } else { "0".into() },
        "session_windows" => app.windows.len().to_string(),
        "window_index" => (win_idx + app.window_base_index).to_string(),
        "window_name" => win.name.clone(),
        "window_active" => if win_idx == app.active_idx { "1".into() } else { "0".into() },
        "window_panes" => count_panes(&win.root).to_string(),
        "window_flags" => {
            if win_idx == app.active_idx { "*".into() }
            else if win_idx == app.last_window_idx { "-".into() }
            else { String::new() }
        }
        "pane_index" => {
            get_active_pane_id(&win.root, &win.active_path).unwrap_or(0).to_string()
        }
        "pane_title" => win.name.clone(),
        "pane_width" => {
            if let Some(p) = active_pane(&win.root, &win.active_path) { p.last_cols.to_string() } else { "0".into() }
        }
        "pane_height" => {
            if let Some(p) = active_pane(&win.root, &win.active_path) { p.last_rows.to_string() } else { "0".into() }
        }
        "pane_active" => "1".to_string(),
        "pane_current_command" => String::new(),
        "pane_pid" => {
            if let Some(p) = active_pane(&win.root, &win.active_path) {
                p.child_pid.map(|pid| pid.to_string()).unwrap_or_default()
            } else { String::new() }
        }
        "host" | "hostname" => hostname_cached(),
        "host_short" => {
            let h = hostname_cached();
            h.split('.').next().unwrap_or(&h).to_string()
        }
        "pid" => std::process::id().to_string(),
        "version" => VERSION.to_string(),
        "mouse" => if app.mouse_enabled { "on".into() } else { "off".into() },
        "prefix" => format_key_binding(&app.prefix_key),
        "status" => if app.status_visible { "on".into() } else { "off".into() },
        _ => String::new(),
    }
}

/// Cached hostname lookup — called frequently in format expansion.
fn hostname_cached() -> String {
    use std::sync::OnceLock;
    static HOSTNAME: OnceLock<String> = OnceLock::new();
    HOSTNAME.get_or_init(|| {
        env::var("COMPUTERNAME")
            .or_else(|_| env::var("HOSTNAME"))
            .unwrap_or_default()
    }).clone()
}
