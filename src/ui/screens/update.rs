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
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::update::TuiUpdateState;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(area);

    let header = Paragraph::new(Line::from(ratatui::text::Span::styled(
        "Update",
        Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
    )))
    .style(Style::default().bg(theme.bg()).fg(theme.fg()))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.sub())),
    );
    f.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    match &app.update_state {
        TuiUpdateState::Idle => {
            lines.push(Line::from("Check for updates?"));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "enter check   esc back",
                Style::default().fg(theme.sub()),
            )));
        }
        TuiUpdateState::Checking => {
            lines.push(Line::from("Checking for updates..."));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "esc back",
                Style::default().fg(theme.sub())
            )));
        }
        TuiUpdateState::UpToDate { current } => {
            lines.push(Line::from(format!("typoist is up to date (v{}).", current)));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "esc back",
                Style::default().fg(theme.sub()),
            )));
        }
        TuiUpdateState::Available { current, latest } => {
            lines.push(Line::from(format!("Update available: v{} -> v{}", current, latest)));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "enter install   esc back",
                Style::default().fg(theme.sub()),
            )));
        }
        TuiUpdateState::Installing => {
            lines.push(Line::from("Installing..."));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "do not close the terminal",
                Style::default().fg(theme.sub()),
            )));
        }
        TuiUpdateState::Done { from, to } => {
            lines.push(Line::from(format!("Updated v{} -> v{}", from, to)));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "restart typoist to use the new version   esc back",
                Style::default().fg(theme.sub()),
            )));
        }
        TuiUpdateState::Error(msg) => {
            lines.push(Line::from(ratatui::text::Span::styled(
                "Update failed",
                Style::default().fg(theme.incorrect()).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(msg.clone()));
            lines.push(Line::from(""));
            lines.push(Line::from(ratatui::text::Span::styled(
                "esc back",
                Style::default().fg(theme.sub()),
            )));
        }
    }

    let body = Paragraph::new(lines)
        .style(Style::default().bg(theme.bg()).fg(theme.fg()))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent()))
                .title(" update "),
        );
    f.render_widget(body, chunks[1]);
}
