use std::io;

use serde::{Serialize, Deserialize};

use crate::types::*;

pub fn infer_title_from_prompt(screen: &vt100::Screen, rows: u16, cols: u16) -> Option<String> {
    let mut last: Option<String> = None;
    for r in (0..rows).rev() {
        let mut s = String::new();
        for c in 0..cols { if let Some(cell) = screen.cell(r, c) { s.push_str(&cell.contents().to_string()); } else { s.push(' '); } }
        let t = s.trim_end().to_string();
        if !t.trim().is_empty() { last = Some(t); break; }
    }
    let Some(line) = last else { return None };
    let trimmed = line.trim().to_string();
    if let Some(pos) = trimmed.rfind('>') {
        let before = trimmed[..pos].trim().to_string();
        if before.contains("\\") || before.contains("/") {
            let parts: Vec<&str> = before.trim_matches(|ch: char| ch == '"').split(['\\','/']).collect();
            if let Some(base) = parts.last() { return Some(base.to_string()); }
        }
        return Some(before);
    }
    if let Some(pos) = trimmed.rfind('$') { return Some(trimmed[..pos].trim().to_string()); }
    if let Some(pos) = trimmed.rfind('#') { return Some(trimmed[..pos].trim().to_string()); }
    Some(trimmed)
}

// resolve_last_session_name and resolve_default_session_name are in session.rs

#[derive(Serialize, Deserialize)]
pub struct WinInfo { pub id: usize, pub name: String, pub active: bool }

#[derive(Serialize, Deserialize)]
pub struct PaneInfo { pub id: usize, pub title: String }

#[derive(Serialize, Deserialize)]
pub struct WinTree { pub id: usize, pub name: String, pub active: bool, pub panes: Vec<PaneInfo> }

pub fn list_windows_json(app: &AppState) -> io::Result<String> {
    let mut v: Vec<WinInfo> = Vec::new();
    for (i, w) in app.windows.iter().enumerate() { v.push(WinInfo { id: w.id, name: w.name.clone(), active: i == app.active_idx }); }
    let s = serde_json::to_string(&v).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("json error: {e}")))?;
    Ok(s)
}

pub fn list_tree_json(app: &AppState) -> io::Result<String> {
    fn collect_panes(node: &Node, out: &mut Vec<PaneInfo>) {
        match node {
            Node::Leaf(p) => { out.push(PaneInfo { id: p.id, title: p.title.clone() }); }
            Node::Split { children, .. } => { for c in children.iter() { collect_panes(c, out); } }
        }
    }
    let mut v: Vec<WinTree> = Vec::new();
    for (i, w) in app.windows.iter().enumerate() {
        let mut panes = Vec::new();
        collect_panes(&w.root, &mut panes);
        v.push(WinTree { id: w.id, name: w.name.clone(), active: i == app.active_idx, panes });
    }
    let s = serde_json::to_string(&v).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("json error: {e}")))?;
    Ok(s)
}

pub const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &str) -> String {
    let bytes = data.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        result.push(BASE64_CHARS[b0 >> 2] as char);
        result.push(BASE64_CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Lookup table for O(1) base64 decoding (maps ASCII byte to 6-bit value, 0xFF = invalid).
const BASE64_DECODE_TABLE: [u8; 128] = {
    let mut table = [0xFFu8; 128];
    let mut i = 0usize;
    while i < 64 {
        table[BASE64_CHARS[i] as usize] = i as u8;
        i += 1;
    }
    table
};

fn base64_char_to_val(b: u8) -> Option<u8> {
    if b >= 128 { return None; }
    let v = BASE64_DECODE_TABLE[b as usize];
    if v == 0xFF { None } else { Some(v) }
}

pub fn base64_decode(encoded: &str) -> Option<String> {
    let mut result = Vec::new();
    let chars: Vec<u8> = encoded.bytes().filter(|&b| b != b'=').collect();
    for chunk in chars.chunks(4) {
        if chunk.len() < 2 { break; }
        let b0 = base64_char_to_val(chunk[0])?;
        let b1 = base64_char_to_val(chunk[1])?;
        result.push((b0 << 2) | (b1 >> 4));
        if chunk.len() > 2 {
            let b2 = base64_char_to_val(chunk[2])?;
            result.push((b1 << 4) | (b2 >> 2));
            if chunk.len() > 3 {
                let b3 = base64_char_to_val(chunk[3])?;
                result.push((b2 << 6) | b3);
            }
        }
    }
    String::from_utf8(result).ok()
}

pub fn color_to_name(c: vt100::Color) -> String {
    match c {
        vt100::Color::Default => "default".to_string(),
        vt100::Color::Idx(i) => format!("idx:{}", i),
        vt100::Color::Rgb(r,g,b) => format!("rgb:{},{},{}", r,g,b),
    }
}
