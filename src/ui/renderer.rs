/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░░███╔═╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░██╔══╝░░░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗███████╗██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚══════╝╚═╝░╚════╝░

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

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let bg = Style::default().bg(theme.bg()).fg(theme.fg());
    // paint the whole screen first so there's no terminal flicker showing through
    f.render_widget(Block::default().style(bg), f.size());

    match app.screen {
        Screen::Test => draw_test(f, app),
        Screen::Results => draw_results(f, app),
        Screen::ThemePicker => draw_theme_picker(f, app),
        Screen::Help => draw_help(f, app),
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
            "typoist",
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

    let lines = widgets::build_test_lines(&app.test, theme);
    let body = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left)
        .block(Block::default().padding(ratatui::widgets::Padding::new(4, 4, 2, 2)));
    f.render_widget(body, areas.body);

    // footer: live stats plus a compact cheat sheet
    let stats_line = Line::from(widgets::stat_spans(
        &app.test,
        theme,
        app.settings.show_live_wpm,
        app.settings.show_accuracy,
    ));
    let hint = Line::from(vec![
        Span::styled("tab", Style::default().fg(theme.accent())),
        Span::styled(" restart   ", Style::default().fg(theme.sub())),
        Span::styled("ctrl+h", Style::default().fg(theme.accent())),
        Span::styled(" help   ", Style::default().fg(theme.sub())),
        Span::styled("ctrl+t", Style::default().fg(theme.accent())),
        Span::styled(" theme   ", Style::default().fg(theme.sub())),
        Span::styled("ctrl+w", Style::default().fg(theme.accent())),
        Span::styled(" words   ", Style::default().fg(theme.sub())),
        Span::styled("ctrl+n", Style::default().fg(theme.accent())),
        Span::styled(" mode   ", Style::default().fg(theme.sub())),
        Span::styled("esc, ctrl+c, ctrl+q", Style::default().fg(theme.accent())),
        Span::styled(" quit", Style::default().fg(theme.sub())),
    ]);

    let footer = Paragraph::new(vec![stats_line, hint])
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center);

    f.render_widget(footer, areas.footer);
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
            "typoist — help",
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
        Line::from("  ctrl+h    this help  (or F1)"),
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
