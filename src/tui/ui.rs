use ratatui::{prelude::*, widgets::*};

use super::app::{App, BrowseItem, Mode, SearchMode};

// ── Column layout ──────────────────────────────────────────────────────────────
//   Col 1  [N]       6 chars  – number shortcut
//   Col 2  command  CMD_COL   – "$  <cmd>" truncated with "…"
//   Col 3  desc     remaining – description, word-wrapped, max 3 lines
//
// Column spacing between cols: 1 char (Table::column_spacing(1)).
// At 80-col terminal  → cmd≈38 chars, desc≈26 chars
// At 120-col terminal → cmd≈48 chars, desc≈60 chars
const NUM_COL: u16 = 6;

// ── Entry point ────────────────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header / key hint
            Constraint::Length(3), // search box
            Constraint::Min(0),    // main table (browse or search results)
            Constraint::Length(1), // status bar
        ])
        .split(area);

    render_header(frame, chunks[0], app);
    render_search_box(frame, chunks[1], app);
    match app.mode {
        Mode::Browse => render_browse(frame, chunks[2], app),
        Mode::Search => render_search_results(frame, chunks[2], app),
    }
    render_status_bar(frame, chunks[3], app);
}

// ── Header ─────────────────────────────────────────────────────────────────────

fn render_header(frame: &mut Frame, area: Rect, _app: &App) {
    let hint = "[num/↑↓/Enter] select  [/] search  [//] exact  [q/ESC] back/quit  [Ctrl+C] exit";
    frame.render_widget(
        Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        area,
    );
}

// ── Search box ─────────────────────────────────────────────────────────────────

fn render_search_box(frame: &mut Frame, area: Rect, app: &App) {
    let mode_label = match app.mode {
        Mode::Browse => "",
        Mode::Search => match app.search_mode {
            SearchMode::Fuzzy => " fuzzy ",
            SearchMode::Exact => " exact ",
        },
    };

    let query_text = format!("🔍 {}", app.search_query);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(" search ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mode_len = mode_label.chars().count() as u16;
    if !mode_label.is_empty() && inner.width > mode_len + 2 {
        let splits = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(mode_len)])
            .split(inner);
        frame.render_widget(
            Paragraph::new(query_text).style(Style::default().fg(Color::White)),
            splits[0],
        );
        frame.render_widget(
            Paragraph::new(mode_label)
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Right),
            splits[1],
        );
    } else {
        frame.render_widget(
            Paragraph::new(query_text).style(Style::default().fg(Color::White)),
            inner,
        );
    }
}

// ── Browse table ───────────────────────────────────────────────────────────────

fn render_browse(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.breadcrumb.is_empty() {
        " / ".to_string()
    } else {
        format!(" {} ", app.breadcrumb.join(" > "))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(title);

    if app.items.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("  (empty folder)")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let (cmd_col, desc_width) = col_widths(area);

    let rows: Vec<Row> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let num = num_label(i);
            let selected = i == app.selected_index;
            match item {
                BrowseItem::Folder(f) => {
                    let (ns, cs) = if selected {
                        (
                            Style::default().fg(Color::DarkGray).bg(Color::Blue),
                            Style::default().fg(Color::Black).bg(Color::Blue),
                        )
                    } else {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::Cyan),
                        )
                    };
                    Row::new([
                        Cell::from(num).style(ns),
                        Cell::from(Span::styled(format!("📁  {}", f.name), cs)),
                        Cell::from(""),
                    ])
                    .height(1)
                }
                BrowseItem::Command(c) => {
                    let (ns, cmd_s, desc_s) = if selected {
                        (
                            Style::default().fg(Color::DarkGray).bg(Color::Green),
                            Style::default().fg(Color::Black).bg(Color::Green),
                            Style::default().fg(Color::Black).bg(Color::Green),
                        )
                    } else {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::White),
                            Style::default().fg(Color::DarkGray),
                        )
                    };
                    let cmd_str = truncate(&format!("$  {}", c.cmd), cmd_col as usize);
                    let wrapped = wrap_text(&c.desc, desc_width, 3);
                    let height = wrapped.len().max(1) as u16;
                    let desc_text = lines_to_text(&wrapped, desc_s);
                    Row::new([
                        Cell::from(num).style(ns),
                        Cell::from(Span::styled(cmd_str, cmd_s)),
                        Cell::from(desc_text),
                    ])
                    .height(height)
                }
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(NUM_COL),
            Constraint::Length(cmd_col),
            Constraint::Min(0),
        ],
    )
    .column_spacing(1)
    .block(block);

    frame.render_widget(table, area);
}

// ── Search-results table ───────────────────────────────────────────────────────

fn render_search_results(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!(" results: {} ", app.search_results.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(title);

    if app.search_results.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("  no results — try a different query")
                .style(Style::default().fg(Color::DarkGray)),
            inner,
        );
        return;
    }

    let (cmd_col, desc_width) = col_widths(area);

    let rows: Vec<Row> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let num = num_label(i);
            let selected = i == app.search_selected;

            let (ns, cmd_s, desc_s, meta_s) = if selected {
                (
                    Style::default().fg(Color::DarkGray).bg(Color::Green),
                    Style::default().fg(Color::Black).bg(Color::Green),
                    Style::default().fg(Color::Black).bg(Color::Green),
                    Style::default().fg(Color::Black).bg(Color::Green),
                )
            } else {
                (
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::White),
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::DarkGray),
                )
            };

            let cmd_str = truncate(&format!("$  {}", result.command.cmd), cmd_col as usize);

            // Desc column: up to 2 wrapped desc lines + 1 meta line (path + tags)
            let mut desc_lines = wrap_text(&result.command.desc, desc_width, 2);

            // Build meta line: [folder/path]  #tag1 #tag2
            let mut meta_parts: Vec<String> = Vec::new();
            if !result.folder_path.is_empty() {
                meta_parts.push(format!("[{}]", result.folder_path.join("/")));
            }
            if !result.command.tags.is_empty() {
                let tags: String = result
                    .command
                    .tags
                    .iter()
                    .map(|t| format!("#{}", t))
                    .collect::<Vec<_>>()
                    .join(" ");
                meta_parts.push(tags);
            }
            let meta_str = meta_parts.join("  ");

            // Build Text with styled lines
            let mut lines: Vec<Line> = desc_lines
                .iter_mut()
                .map(|l| Line::from(Span::styled(l.clone(), desc_s)))
                .collect();

            if !meta_str.is_empty() && lines.len() < 3 {
                lines.push(Line::from(Span::styled(meta_str, meta_s)));
            }

            let height = lines.len().max(1) as u16;
            let desc_text = Text::from(lines);

            Row::new([
                Cell::from(num).style(ns),
                Cell::from(Span::styled(cmd_str, cmd_s)),
                Cell::from(desc_text),
            ])
            .height(height)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(NUM_COL),
            Constraint::Length(cmd_col),
            Constraint::Min(0),
        ],
    )
    .column_spacing(1)
    .block(block);

    frame.render_widget(table, area);
}

// ── Status bar ─────────────────────────────────────────────────────────────────

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let text = match app.mode {
        Mode::Browse => {
            let folders = app
                .items
                .iter()
                .filter(|i| matches!(i, BrowseItem::Folder(_)))
                .count();
            let cmds = app
                .items
                .iter()
                .filter(|i| matches!(i, BrowseItem::Command(_)))
                .count();
            format!(
                " 📁 {}  ⚡ {}  |  {} total commands",
                folders,
                cmds,
                app.store.commands.len()
            )
        }
        Mode::Search => format!(
            " query: \"{}\"  |  {} results  |  {} total",
            app.effective_query(),
            app.search_results.len(),
            app.store.commands.len()
        ),
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Compute (cmd_col_width, desc_text_wrap_width) based on available area.
/// cmd_col covers "$  " prefix + truncated command text.
/// desc_width is the character budget for word-wrapping descriptions.
fn col_widths(area: Rect) -> (u16, usize) {
    // inner width = area width minus 2 border chars
    let inner = area.width.saturating_sub(2);
    // available for cmd + desc columns (subtract num col + 2 column spacers)
    let avail = inner.saturating_sub(NUM_COL + 2);
    // cmd column: ~52% of available, clamped to [20, 52]
    let cmd_col = ((avail as f32 * 0.52).round() as u16).clamp(20, 52);
    // desc wrap width: remaining available chars (subtract cmd col, no extra spacers needed)
    let desc_width = avail.saturating_sub(cmd_col) as usize;
    let desc_width = desc_width.max(8);
    (cmd_col, desc_width)
}

/// Number label for a row: "[1]"–"[9]", "[0]" for the 10th, blank beyond that.
fn num_label(idx: usize) -> String {
    if idx < 9 {
        format!(" [{}] ", idx + 1)
    } else if idx == 9 {
        " [0] ".to_string()
    } else {
        "      ".to_string() // 6 spaces = NUM_COL
    }
}

/// Truncate text to at most `max_chars` characters, appending "…" if cut.
fn truncate(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        text.to_string()
    } else {
        let mut s: String = chars[..max_chars.saturating_sub(1)].iter().collect();
        s.push('…');
        s
    }
}

/// Word-wrap `text` into lines of at most `max_width` chars, capped at `max_lines`.
fn wrap_text(text: &str, max_width: usize, max_lines: usize) -> Vec<String> {
    if text.is_empty() || max_width == 0 || max_lines == 0 {
        return vec![];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut cur_len: usize = 0;

    for word in text.split_whitespace() {
        let wlen = word.chars().count();
        if cur_len == 0 {
            // First word on a new line — truncate if the single word is wider than max_width
            let placed = truncate(word, max_width);
            cur_len = placed.chars().count();
            current = placed;
        } else if cur_len + 1 + wlen <= max_width {
            current.push(' ');
            current.push_str(word);
            cur_len += 1 + wlen;
        } else {
            // Flush current line
            lines.push(current.clone());
            if lines.len() >= max_lines {
                return lines;
            }
            current.clear();
            let placed = truncate(word, max_width);
            cur_len = placed.chars().count();
            current = placed;
        }
    }

    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    lines
}

/// Convert a slice of strings into a ratatui `Text` with the given style on each line.
fn lines_to_text<'a>(lines: &[String], style: Style) -> Text<'a> {
    if lines.is_empty() {
        return Text::default();
    }
    Text::from(
        lines
            .iter()
            .map(|l| Line::from(Span::styled(l.clone(), style)))
            .collect::<Vec<_>>(),
    )
}
