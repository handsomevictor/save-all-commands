use ratatui::{
    prelude::*,
    widgets::*,
};

use super::app::{App, BrowseItem, Mode, SearchMode};

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Layout: header, search_box, main area, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(3), // search box
            Constraint::Min(0),    // main area
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

fn render_header(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "[数字] 直接选择  [↑↓] 移动  [Enter] 确认  [q/ESC] 返回/退出";
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn render_search_box(frame: &mut Frame, area: Rect, app: &App) {
    let mode_label = match app.mode {
        Mode::Search => match app.search_mode {
            SearchMode::Fuzzy => " 模糊搜索 ",
            SearchMode::Exact => " 精确搜索 ",
        },
        Mode::Browse => "",
    };

    let search_icon = "🔍 ";
    let query_text = &app.search_query;

    let left_text = format!("{}{}", search_icon, query_text);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title("搜索");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: left for query, right for mode label
    let mode_label_len = mode_label.chars().count() as u16;
    if inner.width > mode_label_len + 2 {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(mode_label_len),
            ])
            .split(inner);

        let query_paragraph = Paragraph::new(left_text)
            .style(Style::default().fg(Color::White));
        frame.render_widget(query_paragraph, h_chunks[0]);

        if !mode_label.is_empty() {
            let label_paragraph = Paragraph::new(mode_label)
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Right);
            frame.render_widget(label_paragraph, h_chunks[1]);
        }
    } else {
        let query_paragraph = Paragraph::new(left_text)
            .style(Style::default().fg(Color::White));
        frame.render_widget(query_paragraph, inner);
    }
}

fn render_browse(frame: &mut Frame, area: Rect, app: &App) {
    let folders: Vec<&BrowseItem> = app
        .items
        .iter()
        .filter(|i| matches!(i, BrowseItem::Folder(_)))
        .collect();
    let commands: Vec<&BrowseItem> = app
        .items
        .iter()
        .filter(|i| matches!(i, BrowseItem::Command(_)))
        .collect();

    // Build list items
    let mut list_items: Vec<ListItem> = Vec::new();

    if !folders.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(" 📁 文件夹 ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])));

        for (i, item) in folders.iter().enumerate() {
            if let BrowseItem::Folder(f) = item {
                let num = if i < 9 { (i + 1).to_string() } else { "0".to_string() };
                let global_idx = i; // folders are first in items
                let is_selected = app.selected_index == global_idx;
                let style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Blue)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                let line = Line::from(vec![
                    Span::styled(format!(" [{}] ", num), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("📁 {}", f.name), style),
                ]);
                list_items.push(ListItem::new(line));
            }
        }
    }

    if !folders.is_empty() && !commands.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("─────────────────────────────────────", Style::default().fg(Color::DarkGray)),
        ])));
    }

    if !commands.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(" ⚡ 命令 ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])));

        let folder_count = folders.len();
        for (i, item) in commands.iter().enumerate() {
            if let BrowseItem::Command(c) = item {
                let num = if i < 9 { (i + 1).to_string() } else { "0".to_string() };
                let global_idx = folder_count + i;
                let is_selected = app.selected_index == global_idx;
                let cmd_style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                let desc_style = if is_selected {
                    Style::default().fg(Color::Black).bg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let desc_part = if c.desc.is_empty() {
                    String::new()
                } else {
                    format!("  # {}", c.desc)
                };

                let line = Line::from(vec![
                    Span::styled(format!(" [{}] ", num), Style::default().fg(Color::DarkGray)),
                    Span::styled(c.cmd.clone(), cmd_style),
                    Span::styled(desc_part, desc_style),
                ]);
                list_items.push(ListItem::new(line));
            }
        }
    }

    if app.items.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("  (空)", Style::default().fg(Color::DarkGray)),
        ])));
    }

    // Build breadcrumb title
    let breadcrumb_title = if app.breadcrumb.is_empty() {
        " 根目录 ".to_string()
    } else {
        format!(" {} ", app.breadcrumb.join(" > "))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(breadcrumb_title);

    let list = List::new(list_items).block(block);
    frame.render_widget(list, area);
}

fn render_search_results(frame: &mut Frame, area: Rect, app: &App) {
    let mut list_items: Vec<ListItem> = Vec::new();

    for (i, result) in app.search_results.iter().enumerate() {
        let num = if i < 9 {
            (i + 1).to_string()
        } else if i == 9 {
            "0".to_string()
        } else {
            " ".to_string()
        };
        let is_selected = app.search_selected == i;

        let cmd_style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        let meta_style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let path_str = if result.folder_path.is_empty() {
            String::new()
        } else {
            format!(" [{}]", result.folder_path.join("/"))
        };

        let desc_part = if result.command.desc.is_empty() {
            String::new()
        } else {
            format!("  # {}", result.command.desc)
        };

        let line = Line::from(vec![
            Span::styled(format!(" [{}] ", num), Style::default().fg(Color::DarkGray)),
            Span::styled(result.command.cmd.clone(), cmd_style),
            Span::styled(desc_part, meta_style),
            Span::styled(path_str, meta_style),
        ]);
        list_items.push(ListItem::new(line));
    }

    if app.search_results.is_empty() {
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled("  无结果", Style::default().fg(Color::DarkGray)),
        ])));
    }

    let title = format!(" 搜索结果: {} 条 ", app.search_results.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(title);

    let list = List::new(list_items).block(block);
    frame.render_widget(list, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let status = match app.mode {
        Mode::Browse => {
            let folder_count = app.items.iter().filter(|i| matches!(i, BrowseItem::Folder(_))).count();
            let cmd_count = app.items.iter().filter(|i| matches!(i, BrowseItem::Command(_))).count();
            format!(
                " 📁 {} 个文件夹  ⚡ {} 个命令  |  共 {} 个命令",
                folder_count,
                cmd_count,
                app.store.commands.len()
            )
        }
        Mode::Search => {
            format!(
                " 搜索: \"{}\"  |  {} 条结果  |  共 {} 个命令",
                app.search_query,
                app.search_results.len(),
                app.store.commands.len()
            )
        }
    };

    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Reset));
    frame.render_widget(paragraph, area);
}
