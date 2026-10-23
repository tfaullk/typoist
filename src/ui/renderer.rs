/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗         ██╗   ███████╗    ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝        ███║   ██╔════╝   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║           ╚██║   ███████╗   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║            ██║   ╚════██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║            ██║██╗███████║██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝            ╚═╝╚═╝╚══════╝╚═╝ ╚═════╝

Made with ♥ by tfaullk

*/


use ratatui::layout::Alignment;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Screen};
use crate::themes::ThemeWarning;
use crate::ui::{layout, widgets};
use crate::ui::screens;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let bg = Style::default().bg(theme.bg()).fg(theme.fg());
    // paint the whole screen first so there's no terminal flicker showing through
    f.render_widget(Block::default().style(bg), f.size());

    match app.screen {
        Screen::Menu => screens::menu::draw(f, app),
        Screen::Test => draw_test(f, app),
        Screen::Results => draw_results(f, app),
        Screen::ThemePicker => draw_theme_picker(f, app),
        Screen::Help => draw_help(f, app),
        Screen::Update => screens::update::draw(f, app),
    }
    // error popup sits on top of whatever screen is showing
    if let Some(warning) = app.current_theme_warning() {
        draw_theme_warning_popup(f, app, warning);
    } 
}

fn draw_test(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let areas = layout::split(f.size());

    // header: app name, mode, word set
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("typoist {}", env!("CARGO_PKG_VERSION")),
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(app.test.mode.label(), Style::default().fg(theme.sub())),
        Span::raw("  "),
        Span::styled(
            app.word_set_display_name(),
            Style::default().fg(theme.sub()),
        ),
    ]))
    .style(Style::default().bg(theme.bg()).fg(theme.fg()))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.sub())),
    );
    f.render_widget(header, areas.header);

    // if the wpm graph is on, split the body vertically so the graph
    // sits above the text
    let (graph_area, text_area) = if app.settings.show_wpm_graph {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(1),
            ])
            .split(areas.body);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, areas.body)
    };

    // FIX: bind the area to a local before rendering so we can read its width,
    // then use that local for both the widget call and the width calc
    if let Some(graph_rect) = graph_area {
        let graph_width = graph_rect.width.saturating_sub(2) as usize;
        let graph_lines = widgets::build_wpm_graph(&app.test, theme, graph_width);
        let graph = Paragraph::new(graph_lines)
            .style(Style::default().bg(theme.bg()).fg(theme.fg()))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.sub())),
            );
        f.render_widget(graph, graph_rect);
    }
    
    let text_width = text_area.width.saturating_sub(8) as usize;
    let lines = widgets::build_test_lines(&app.test, theme, text_width);
    let body = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left)
        .block(Block::default().padding(ratatui::widgets::Padding::new(4, 4, 2, 2)));
    f.render_widget(body, text_area);

    let stats_line = Line::from(widgets::stat_spans(
        &app.test,
        theme,
        app.settings.show_live_wpm,
        app.settings.show_accuracy,
    ));

    let hint = Line::from(vec![
        Span::styled("ctrl+h", Style::default().fg(theme.accent())),
        Span::styled(" keybinds/help", Style::default().fg(theme.sub())),
    ]);

    let footer_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(areas.footer);

    let sep_top = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.sub()));
    f.render_widget(sep_top, footer_chunks[1]);

    let stats_cols = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(30),
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Percentage(30),
        ])
        .split(footer_chunks[3]);

    let current_pb = app.settings.get_pb(&app.test.mode.label());

    let pb_widget = if current_pb > 0.0 {
        Paragraph::new(Line::from(vec![
            Span::styled("PB: ", Style::default().fg(theme.sub())),
            Span::styled(
                format!("{:.0}", current_pb),
                Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
            ),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("PB: ", Style::default().fg(theme.sub())),
            Span::styled("--", Style::default().fg(theme.sub())),
        ]))
    };

    let pb_widget = pb_widget
        .style(Style::default().bg(theme.bg()))
        .alignment(Alignment::Left)
        .block(Block::default().padding(ratatui::widgets::Padding::new(2, 0, 0, 0)));

    f.render_widget(pb_widget, stats_cols[0]);

    let stats_widget = Paragraph::new(stats_line)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center);
    f.render_widget(stats_widget, stats_cols[1]);

    let sep_bottom = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.sub()));
    f.render_widget(sep_bottom, footer_chunks[5]);

    let hint_widget = Paragraph::new(hint)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center);
    f.render_widget(hint_widget, footer_chunks[7]);

    let sep_bottom = Block::default() 
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme.sub()));
    f.render_widget(sep_bottom, footer_chunks[8]);
  }


fn draw_results(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    let stats = &app.test.stats;
    let lines = vec![
        Line::from(Span::styled(
            "Test Complete",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("WPM: {:.1}", stats.wpm())),
        Line::from(format!("Raw WPM: {:.1}", stats.raw_wpm())),
        Line::from(format!("Accuracy: {:.1}%", stats.accuracy())),
        Line::from(format!("Consistency: {:.1}%", stats.consistency())),
        Line::from(""),
        Line::from(Span::styled(
            "Press Tab to restart • Esc to return",
            Style::default().fg(theme.sub()),
        )),
    ];

    let p = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent()))
                .title(" results "),
        );

    let area = centered_rect(60, 50, f.size());

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(p, area);
}

fn draw_theme_picker(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let names = &app.themes;
    let mut lines = vec![
        Line::from(Span::styled(
            "Select theme (↑/↓ to preview, enter to confirm, esc to cancel)",
            Style::default().fg(theme.sub()),
        )),
        Line::from(""),
    ];
    for (i, theme_item) in names.iter().enumerate() {
        let selected = i == app.theme_index;
        let name = &theme_item.name;
        let style = if selected {
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(theme.fg())
        };
        lines.push(Line::from(Span::styled(format!("  {}  ", name), style)));
    }

    let p = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent()))
                .title(" themes "),
        );
    f.render_widget(ratatui::widgets::Clear, centered_rect(60, 60, f.size()));
    f.render_widget(p, centered_rect(60, 60, f.size()));
}

fn draw_help(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let lines = vec![
        Line::from(Span::styled(
            "typoist — keybinds",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  tab       restart test (new words)"),
        Line::from("  esc       reset current test  •  quit when idle/on results"),
        Line::from(""),
        Line::from("  ctrl+t    open theme picker"),
        Line::from("  ctrl+w    cycle word set"),
        Line::from("  ctrl+n    cycle mode"),
        Line::from("  ctrl+l    toggle live WPM"),
        Line::from("  ctrl+a    toggle accuracy display"),
        Line::from("  ctrl+g    toggle live WPM graph"),
        Line::from("  ctrl+h    this keybind/help menu  (or F1)"),
        Line::from(""),
        Line::from(Span::styled(
            "  press any key to close",
            Style::default().fg(theme.sub()),
        )),
    ];
    let p = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent())),
        );
    f.render_widget(ratatui::widgets::Clear, centered_rect(60, 60, f.size()));
    f.render_widget(p, centered_rect(60, 60, f.size()));
}

// the usual "cut a centered box out of the frame" helper, px/py are percentages
fn centered_rect(px: u16, py: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let vertical = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - py) / 2),
            ratatui::layout::Constraint::Percentage(py),
            ratatui::layout::Constraint::Percentage((100 - py) / 2),
        ])
        .split(r);
    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - px) / 2),
            ratatui::layout::Constraint::Percentage(px),
            ratatui::layout::Constraint::Percentage((100 - px) / 2),
        ])
        .split(vertical[1])[1]
}

fn draw_theme_warning_popup(f: &mut Frame, app: &App, warning: &ThemeWarning) {
    let theme = &app.theme;
    let warning_count = app.theme_warnings.len();
    let warning_number = app.theme_warning_index + 1;

    let mut lines = vec![
        Line::from(Span::styled(
            &warning.title,
            Style::default()
                .fg(theme.incorrect())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for line in warning.message.lines() {
        lines.push(Line::from(line.to_string()));
    }

    if let Some(path) = &warning.path {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "File:",
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(path.clone()));
    }

    if let Some(details) = &warning.details {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Technical details:",
            Style::default().fg(theme.sub()),
        )));
        lines.push(Line::from(Span::styled(
            details.clone(),
            Style::default().fg(theme.sub()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Your other themes are still available",
        Style::default().fg(theme.sub()),
    )));
    lines.push(Line::from(""));

    let dismiss_hint = if warning_count > 1 {
        format!(
            "Press Enter, Esc, Space, ↓, or → for next warning ({}/{})",
            warning_number,
            warning_count
        )
    } else {
        String::from("Press Enter, Esc, Space, ↓, or → to continue")
    };

    lines.push(Line::from(Span::styled(
        dismiss_hint,
        Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD),
    )));

    let popup = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.incorrect()))
                .title(" theme warning "),
        );

    let area = centered_rect(78, 65, f.size());

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(popup, area);
}
