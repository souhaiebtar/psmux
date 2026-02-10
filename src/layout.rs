use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use unicode_width::UnicodeWidthStr;

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
pub struct CellJson { pub text: String, pub fg: u32, pub bg: u32, pub flags: u8 }

#[derive(Serialize, Deserialize)]
pub struct CellRunJson {
    pub text: String,
    pub fg: u32,
    pub bg: u32,
    pub flags: u8,
    pub width: u16,
}

#[derive(Serialize, Deserialize)]
pub struct RowRunsJson {
    pub runs: Vec<CellRunJson>,
}

#[derive(Serialize, Deserialize)]
pub struct PaneDeltaJson {
    pub id: usize,
    pub cursor_row: u16,
    pub cursor_col: u16,
    #[serde(default)]
    pub alternate_screen: bool,
    #[serde(default)]
    pub rows_v2: Vec<RowRunsJson>,
}

#[derive(Serialize)]
struct LayoutDeltaJson {
    panes: Vec<PaneDeltaJson>,
}

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
        #[serde(default)]
        alternate_screen: bool,
        active: bool,
        copy_mode: bool,
        scroll_offset: usize,
        sel_start_row: Option<u16>,
        sel_start_col: Option<u16>,
        sel_end_row: Option<u16>,
        sel_end_col: Option<u16>,
        #[serde(default)]
        content: Vec<Vec<CellJson>>,
        #[serde(default)]
        rows_v2: Vec<RowRunsJson>,
    },
}

fn encode_color(c: vt100::Color) -> u32 {
    match c {
        vt100::Color::Default => 0,
        vt100::Color::Idx(i) => 0x01_00_00_00 | i as u32,
        vt100::Color::Rgb(r, g, b) => {
            0x02_00_00_00 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }
    }
}

pub fn dump_layout_json(app: &mut AppState) -> io::Result<String> {
    let (json, _) = dump_layout_json_with_title_changes(app)?;
    Ok(json)
}

pub fn dump_layout_json_with_title_changes(app: &mut AppState) -> io::Result<(String, bool)> {
    let in_copy_mode = matches!(app.mode, Mode::CopyMode);
    let scroll_offset = app.copy_scroll_offset;
    const TITLE_INFER_INTERVAL: Duration = Duration::from_millis(500);

    fn build(
        node: &mut Node,
        cur_path: &mut Vec<usize>,
        active_path: &[usize],
        include_full_content: bool,
        title_changed: &mut bool,
    ) -> LayoutJson {
        match node {
            Node::Split { kind, sizes, children } => {
                let k = match *kind { LayoutKind::Horizontal => "Horizontal".to_string(), LayoutKind::Vertical => "Vertical".to_string() };
                let mut ch: Vec<LayoutJson> = Vec::new();
                for (i, c) in children.iter_mut().enumerate() {
                    cur_path.push(i);
                    ch.push(build(c, cur_path, active_path, include_full_content, title_changed));
                    cur_path.pop();
                }
                LayoutJson::Split { kind: k, sizes: sizes.clone(), children: ch }
            }
            Node::Leaf(p) => {
                const FLAG_DIM: u8 = 1;
                const FLAG_BOLD: u8 = 2;
                const FLAG_ITALIC: u8 = 4;
                const FLAG_UNDERLINE: u8 = 8;
                const FLAG_INVERSE: u8 = 16;

                let Ok(parser) = p.term.lock() else {
                    // Mutex poisoned - return minimal leaf
                    return LayoutJson::Leaf {
                        id: p.id,
                        rows: p.last_rows,
                        cols: p.last_cols,
                        cursor_row: 0,
                        cursor_col: 0,
                        alternate_screen: false,
                        active: false,
                        copy_mode: false,
                        scroll_offset: 0,
                        sel_start_row: None,
                        sel_start_col: None,
                        sel_end_row: None,
                        sel_end_col: None,
                        content: Vec::new(),
                        rows_v2: Vec::new(),
                    };
                };
                let screen = parser.screen();
                let (cr, cc) = screen.cursor_position();
                let alternate_screen = screen.alternate_screen();
                let now = Instant::now();
                let is_active_pane = *cur_path == active_path;
                if is_active_pane
                    && !alternate_screen
                    && now.duration_since(p.last_title_infer_at) >= TITLE_INFER_INTERVAL
                {
                    if let Some(t) = infer_title_from_prompt(&screen, p.last_rows, p.last_cols) {
                        if t != p.title {
                            p.title = t;
                            *title_changed = true;
                        }
                    }
                    p.last_title_infer_at = now;
                }
                let need_full_content = include_full_content && *cur_path == active_path;
                let mut lines: Vec<Vec<CellJson>> = if need_full_content {
                    Vec::with_capacity(p.last_rows as usize)
                } else {
                    Vec::new()
                };
                let mut rows_v2: Vec<RowRunsJson> = Vec::with_capacity(p.last_rows as usize);
                for r in 0..p.last_rows {
                    let mut row: Vec<CellJson> = if need_full_content {
                        Vec::with_capacity(p.last_cols as usize)
                    } else {
                        Vec::new()
                    };
                    let mut runs: Vec<CellRunJson> = Vec::new();
                    let mut c = 0;
                    while c < p.last_cols {
                        let mut text = String::new();
                        let mut fg_code = encode_color(vt100::Color::Default);
                        let mut bg_code = encode_color(vt100::Color::Default);
                        let mut bold = false;
                        let mut italic = false;
                        let mut underline = false;
                        let mut inverse = false;
                        let mut dim = false;
                        if let Some(cell) = screen.cell(r, c) {
                            let fg = cell.fgcolor();
                            let bg = cell.bgcolor();
                            fg_code = encode_color(fg);
                            bg_code = encode_color(bg);
                            text = cell.contents().to_string();
                            if text.is_empty() {
                                text.push(' ');
                            }
                            bold = cell.bold();
                            italic = cell.italic();
                            underline = cell.underline();
                            inverse = cell.inverse();
                            dim = cell.dim();
                        } else {
                            text.push(' ');
                        }

                        let mut width = UnicodeWidthStr::width(text.as_str()) as u16;
                        if width == 0 {
                            width = 1;
                        }

                        let mut flags = 0u8;
                        if dim { flags |= FLAG_DIM; }
                        if bold { flags |= FLAG_BOLD; }
                        if italic { flags |= FLAG_ITALIC; }
                        if underline { flags |= FLAG_UNDERLINE; }
                        if inverse { flags |= FLAG_INVERSE; }

                        if need_full_content {
                            if let Some(last) = runs.last_mut() {
                                if last.fg == fg_code && last.bg == bg_code && last.flags == flags {
                                    last.text.push_str(text.as_str());
                                    last.width = last.width.saturating_add(width);
                                } else {
                                    runs.push(CellRunJson { text: text.clone(), fg: fg_code, bg: bg_code, flags, width });
                                }
                            } else {
                                runs.push(CellRunJson { text: text.clone(), fg: fg_code, bg: bg_code, flags, width });
                            }
                            row.push(CellJson {
                                text,
                                fg: fg_code,
                                bg: bg_code,
                                flags,
                            });
                            for _ in 1..width {
                                row.push(CellJson {
                                    text: String::new(),
                                    fg: fg_code,
                                    bg: bg_code,
                                    flags,
                                });
                            }
                        } else if let Some(last) = runs.last_mut() {
                            if last.fg == fg_code && last.bg == bg_code && last.flags == flags {
                                last.text.push_str(text.as_str());
                                last.width = last.width.saturating_add(width);
                            } else {
                                runs.push(CellRunJson { text, fg: fg_code, bg: bg_code, flags, width });
                            }
                        } else {
                            runs.push(CellRunJson { text, fg: fg_code, bg: bg_code, flags, width });
                        }

                        c = c.saturating_add(width.max(1));
                    }
                    if need_full_content {
                        while row.len() < p.last_cols as usize {
                            row.push(CellJson {
                                text: " ".to_string(),
                                fg: encode_color(vt100::Color::Default),
                                bg: encode_color(vt100::Color::Default),
                                flags: 0,
                            });
                        }
                        lines.push(row);
                    }
                    rows_v2.push(RowRunsJson { runs });
                }
                LayoutJson::Leaf {
                    id: p.id,
                    rows: p.last_rows,
                    cols: p.last_cols,
                    cursor_row: cr,
                    cursor_col: cc,
                    alternate_screen,
                    active: false,
                    copy_mode: false,
                    scroll_offset: 0,
                    sel_start_row: None,
                    sel_start_col: None,
                    sel_end_row: None,
                    sel_end_col: None,
                    content: lines,
                    rows_v2,
                }
            }
        }
    }
    let win = &mut app.windows[app.active_idx];
    let mut path = Vec::new();
    let mut title_changed = false;
    let mut root = build(
        &mut win.root,
        &mut path,
        &win.active_path,
        in_copy_mode,
        &mut title_changed,
    );
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
    Ok((s, title_changed))
}

/// Build pane-only deltas for dirty panes in the active window.
/// Returns (delta_json, title_changed, changed_pane_count).
pub fn dump_panes_delta_json(
    app: &mut AppState,
    dirty_panes: &HashSet<usize>,
) -> io::Result<(String, bool, usize)> {
    const FLAG_DIM: u8 = 1;
    const FLAG_BOLD: u8 = 2;
    const FLAG_ITALIC: u8 = 4;
    const FLAG_UNDERLINE: u8 = 8;
    const FLAG_INVERSE: u8 = 16;
    const TITLE_INFER_INTERVAL: Duration = Duration::from_millis(500);

    fn build_rows_v2(screen: &vt100::Screen, rows: u16, cols: u16) -> Vec<RowRunsJson> {
        let mut rows_v2: Vec<RowRunsJson> = Vec::with_capacity(rows as usize);
        for r in 0..rows {
            let mut runs: Vec<CellRunJson> = Vec::new();
            let mut c = 0;
            while c < cols {
                let mut text = String::new();
                let mut fg_code = encode_color(vt100::Color::Default);
                let mut bg_code = encode_color(vt100::Color::Default);
                let mut bold = false;
                let mut italic = false;
                let mut underline = false;
                let mut inverse = false;
                let mut dim = false;

                if let Some(cell) = screen.cell(r, c) {
                    let fg = cell.fgcolor();
                    let bg = cell.bgcolor();
                    fg_code = encode_color(fg);
                    bg_code = encode_color(bg);
                    text = cell.contents().to_string();
                    if text.is_empty() {
                        text.push(' ');
                    }
                    bold = cell.bold();
                    italic = cell.italic();
                    underline = cell.underline();
                    inverse = cell.inverse();
                    dim = cell.dim();
                } else {
                    text.push(' ');
                }

                let mut width = UnicodeWidthStr::width(text.as_str()) as u16;
                if width == 0 {
                    width = 1;
                }

                let mut flags = 0u8;
                if dim { flags |= FLAG_DIM; }
                if bold { flags |= FLAG_BOLD; }
                if italic { flags |= FLAG_ITALIC; }
                if underline { flags |= FLAG_UNDERLINE; }
                if inverse { flags |= FLAG_INVERSE; }

                if let Some(last) = runs.last_mut() {
                    if last.fg == fg_code && last.bg == bg_code && last.flags == flags {
                        last.text.push_str(text.as_str());
                        last.width = last.width.saturating_add(width);
                    } else {
                        runs.push(CellRunJson { text, fg: fg_code, bg: bg_code, flags, width });
                    }
                } else {
                    runs.push(CellRunJson { text, fg: fg_code, bg: bg_code, flags, width });
                }

                c = c.saturating_add(width.max(1));
            }
            rows_v2.push(RowRunsJson { runs });
        }
        rows_v2
    }

    fn collect_deltas(
        node: &mut Node,
        cur_path: &mut Vec<usize>,
        active_path: &[usize],
        dirty_panes: &HashSet<usize>,
        title_changed: &mut bool,
        out: &mut Vec<PaneDeltaJson>,
    ) {
        match node {
            Node::Split { children, .. } => {
                for (i, child) in children.iter_mut().enumerate() {
                    cur_path.push(i);
                    collect_deltas(
                        child,
                        cur_path,
                        active_path,
                        dirty_panes,
                        title_changed,
                        out,
                    );
                    cur_path.pop();
                }
            }
            Node::Leaf(p) => {
                if !dirty_panes.contains(&p.id) {
                    return;
                }
                let Ok(parser) = p.term.lock() else { return };
                let screen = parser.screen();
                let (cursor_row, cursor_col) = screen.cursor_position();
                let alternate_screen = screen.alternate_screen();
                let now = Instant::now();
                let is_active_pane = *cur_path == active_path;
                if is_active_pane
                    && !alternate_screen
                    && now.duration_since(p.last_title_infer_at) >= TITLE_INFER_INTERVAL
                {
                    if let Some(t) = infer_title_from_prompt(&screen, p.last_rows, p.last_cols) {
                        if t != p.title {
                            p.title = t;
                            *title_changed = true;
                        }
                    }
                    p.last_title_infer_at = now;
                }

                let delta = PaneDeltaJson {
                    id: p.id,
                    cursor_row,
                    cursor_col,
                    alternate_screen,
                    rows_v2: build_rows_v2(&screen, p.last_rows, p.last_cols),
                };
                out.push(delta);
            }
        }
    }

    let win = &mut app.windows[app.active_idx];
    let mut path = Vec::new();
    let mut title_changed = false;
    let mut panes = Vec::new();
    collect_deltas(
        &mut win.root,
        &mut path,
        &win.active_path,
        dirty_panes,
        &mut title_changed,
        &mut panes,
    );
    let changed = panes.len();
    let s = serde_json::to_string(&LayoutDeltaJson { panes })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("json error: {e}")))?;
    Ok((s, title_changed, changed))
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
