use std::io;

use serde::{Serialize, Deserialize};

use crate::types::*;
use crate::tree::*;
use crate::util::infer_title_from_prompt;

pub fn cycle_top_layout(app: &mut AppState) {
    let win = &mut app.windows[app.active_idx];
    // toggle parent of active path, else toggle root
    if !win.active_path.is_empty() {
        let parent_path = &win.active_path[..win.active_path.len()-1].to_vec();
        if let Some(Node::Split { kind, sizes, .. }) = get_split_mut(&mut win.root, &parent_path.to_vec()) {
            *kind = match *kind { LayoutKind::Horizontal => LayoutKind::Vertical, LayoutKind::Vertical => LayoutKind::Horizontal };
            *sizes = vec![50,50];
        }
    } else {
        if let Node::Split { kind, sizes, .. } = &mut win.root { *kind = match *kind { LayoutKind::Horizontal => LayoutKind::Vertical, LayoutKind::Vertical => LayoutKind::Horizontal }; *sizes = vec![50,50]; }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CellJson { pub text: String, pub fg: String, pub bg: String, pub bold: bool, pub italic: bool, pub underline: bool, pub inverse: bool, pub dim: bool }

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LayoutJson {
    #[serde(rename = "split")]
    Split { kind: String, sizes: Vec<u16>, children: Vec<LayoutJson> },
    #[serde(rename = "leaf")]
    Leaf {
        id: usize,
        rows: u16,
        cols: u16,
        cursor_row: u16,
        cursor_col: u16,
        active: bool,
        copy_mode: bool,
        scroll_offset: usize,
        sel_start_row: Option<u16>,
        sel_start_col: Option<u16>,
        sel_end_row: Option<u16>,
        sel_end_col: Option<u16>,
        content: Vec<Vec<CellJson>>,
    },
}

pub fn dump_layout_json(app: &mut AppState) -> io::Result<String> {
    let in_copy_mode = matches!(app.mode, Mode::CopyMode);
    let scroll_offset = app.copy_scroll_offset;
    
    fn build(node: &mut Node) -> LayoutJson {
        match node {
            Node::Split { kind, sizes, children } => {
                let k = match *kind { LayoutKind::Horizontal => "Horizontal".to_string(), LayoutKind::Vertical => "Vertical".to_string() };
                let mut ch: Vec<LayoutJson> = Vec::new();
                for c in children.iter_mut() { ch.push(build(c)); }
                LayoutJson::Split { kind: k, sizes: sizes.clone(), children: ch }
            }
            Node::Leaf(p) => {
                let parser = p.term.lock().unwrap();
                let screen = parser.screen();
                let (cr, cc) = screen.cursor_position();
                if let Some(t) = infer_title_from_prompt(&screen, p.last_rows, p.last_cols) { p.title = t; }
                let mut lines: Vec<Vec<CellJson>> = Vec::new();
                for r in 0..p.last_rows {
                    let mut row: Vec<CellJson> = Vec::new();
                    for c in 0..p.last_cols {
                        if let Some(cell) = screen.cell(r, c) {
                            let fg = crate::util::color_to_name(cell.fgcolor());
                            let bg = crate::util::color_to_name(cell.bgcolor());
                            let text = cell.contents().to_string();
                            row.push(CellJson { text, fg, bg, bold: cell.bold(), italic: cell.italic(), underline: cell.underline(), inverse: cell.inverse(), dim: cell.dim() });
                        } else {
                            row.push(CellJson { text: " ".to_string(), fg: "default".to_string(), bg: "default".to_string(), bold: false, italic: false, underline: false, inverse: false, dim: false });
                        }
                    }
                    lines.push(row);
                }
                LayoutJson::Leaf {
                    id: p.id,
                    rows: p.last_rows,
                    cols: p.last_cols,
                    cursor_row: cr,
                    cursor_col: cc,
                    active: false,
                    copy_mode: false,
                    scroll_offset: 0,
                    sel_start_row: None,
                    sel_start_col: None,
                    sel_end_row: None,
                    sel_end_col: None,
                    content: lines,
                }
            }
        }
    }
    let win = &mut app.windows[app.active_idx];
    let mut root = build(&mut win.root);
    // Mark the active pane and set copy mode info
    fn mark_active(
        node: &mut LayoutJson,
        path: &[usize],
        idx: usize,
        in_copy_mode: bool,
        scroll_offset: usize,
        copy_anchor: Option<(u16, u16)>,
        copy_pos: Option<(u16, u16)>,
    ) {
        match node {
            LayoutJson::Leaf {
                active,
                copy_mode,
                scroll_offset: so,
                sel_start_row,
                sel_start_col,
                sel_end_row,
                sel_end_col,
                ..
            } => {
                let is_active = idx >= path.len();
                *active = is_active;
                if is_active {
                    *copy_mode = in_copy_mode;
                    *so = scroll_offset;
                    if in_copy_mode {
                        if let (Some((ar, ac)), Some((pr, pc))) = (copy_anchor, copy_pos) {
                            *sel_start_row = Some(ar.min(pr));
                            *sel_start_col = Some(ac.min(pc));
                            *sel_end_row = Some(ar.max(pr));
                            *sel_end_col = Some(ac.max(pc));
                        } else {
                            *sel_start_row = None;
                            *sel_start_col = None;
                            *sel_end_row = None;
                            *sel_end_col = None;
                        }
                    } else {
                        *sel_start_row = None;
                        *sel_start_col = None;
                        *sel_end_row = None;
                        *sel_end_col = None;
                    }
                }
            }
            LayoutJson::Split { children, .. } => {
                if idx < path.len() {
                    if let Some(child) = children.get_mut(path[idx]) {
                        mark_active(child, path, idx + 1, in_copy_mode, scroll_offset, copy_anchor, copy_pos);
                    }
                }
            }
        }
    }
    mark_active(
        &mut root,
        &win.active_path,
        0,
        in_copy_mode,
        scroll_offset,
        app.copy_anchor,
        app.copy_pos,
    );
    let s = serde_json::to_string(&root).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("json error: {e}")))?;
    Ok(s)
}

/// Apply a named layout to the current window
pub fn apply_layout(app: &mut AppState, layout: &str) {
    let win = &mut app.windows[app.active_idx];
    
    // Count panes
    fn count_panes(node: &Node) -> usize {
        match node {
            Node::Leaf(_) => 1,
            Node::Split { children, .. } => children.iter().map(count_panes).sum(),
        }
    }
    let pane_count = count_panes(&win.root);
    if pane_count < 2 { return; }
    
    match layout.to_lowercase().as_str() {
        "even-horizontal" | "even-h" => {
            if let Node::Split { kind, sizes, .. } = &mut win.root {
                *kind = LayoutKind::Horizontal;
                let size = 100 / sizes.len().max(1) as u16;
                for s in sizes.iter_mut() { *s = size; }
            }
        }
        "even-vertical" | "even-v" => {
            if let Node::Split { kind, sizes, .. } = &mut win.root {
                *kind = LayoutKind::Vertical;
                let size = 100 / sizes.len().max(1) as u16;
                for s in sizes.iter_mut() { *s = size; }
            }
        }
        "main-horizontal" | "main-h" => {
            if let Node::Split { kind, sizes, .. } = &mut win.root {
                *kind = LayoutKind::Vertical;
                if sizes.len() >= 2 {
                    sizes[0] = 60;
                    let remaining = 40 / (sizes.len() - 1).max(1) as u16;
                    for s in sizes.iter_mut().skip(1) { *s = remaining; }
                }
            }
        }
        "main-vertical" | "main-v" => {
            if let Node::Split { kind, sizes, .. } = &mut win.root {
                *kind = LayoutKind::Horizontal;
                if sizes.len() >= 2 {
                    sizes[0] = 60;
                    let remaining = 40 / (sizes.len() - 1).max(1) as u16;
                    for s in sizes.iter_mut().skip(1) { *s = remaining; }
                }
            }
        }
        "tiled" => {
            if let Node::Split { sizes, .. } = &mut win.root {
                let size = 100 / sizes.len().max(1) as u16;
                for s in sizes.iter_mut() { *s = size; }
            }
        }
        _ => {}
    }
}

/// Cycle through available layouts
pub fn cycle_layout(app: &mut AppState) {
    static LAYOUTS: [&str; 5] = ["even-horizontal", "even-vertical", "main-horizontal", "main-vertical", "tiled"];
    
    let win = &app.windows[app.active_idx];
    let (kind, sizes) = match &win.root {
        Node::Leaf(_) => return,
        Node::Split { kind, sizes, .. } => (*kind, sizes.clone()),
    };
    
    let current_idx = if sizes.is_empty() {
        0
    } else if sizes.iter().all(|s| *s == sizes[0]) {
        match kind {
            LayoutKind::Horizontal => 0,
            LayoutKind::Vertical => 1,
        }
    } else if sizes.len() >= 2 && sizes[0] > sizes[1] {
        match kind {
            LayoutKind::Vertical => 2,
            LayoutKind::Horizontal => 3,
        }
    } else {
        4
    };
    
    let next_idx = (current_idx + 1) % LAYOUTS.len();
    apply_layout(app, LAYOUTS[next_idx]);
}
