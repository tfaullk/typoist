/*


████████╗██╗   ██╗██████╗  ██████╗ ██╗███████╗████████╗         ██╗   ███████╗    ██████╗ 
╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔═══██╗██║██╔════╝╚══██╔══╝        ███║   ██╔════╝   ██╔═████╗
   ██║    ╚████╔╝ ██████╔╝██║   ██║██║███████╗   ██║           ╚██║   ███████╗   ██║██╔██║
   ██║     ╚██╔╝  ██╔═══╝ ██║   ██║██║╚════██║   ██║            ██║   ╚════██║   ████╔╝██║
   ██║      ██║   ██║     ╚██████╔╝██║███████║   ██║            ██║██╗███████║██╗╚██████╔╝
   ╚═╝      ╚═╝   ╚═╝      ╚═════╝ ╚═╝╚══════╝   ╚═╝            ╚═╝╚═╝╚══════╝╚═╝ ╚═════╝

Made with ♥ by tfaullk

*/

use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::ui::banner;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(17),
            Constraint::Min(0),
        ])
        .split(f.size());

    let content_area = chunks[1];

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(9),
        ])
        .split(content_area);

    let banner_lines = banner::lines(theme);
    let banner_widget = Paragraph::new(banner_lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center);
    f.render_widget(banner_widget, inner_chunks[0]);

    let labels = App::menu_labels();
    let mut lines: Vec<Line> = Vec::new();

    for (i, label) in labels.iter().enumerate() {
        let selected = i == app.menu_index;
        let style = if selected {
            Style::default()
                .fg(theme.bg())
                .bg(theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg())
        };

        lines.push(Line::from(Span::styled(
            format!("   {}   ", label),
            style,
        )));

        if i < labels.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓ move   enter select   ctrl+c quit",
        Style::default().fg(theme.sub()),
    )));

    let options = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .alignment(Alignment::Center);
    f.render_widget(options, inner_chunks[1]);
}
