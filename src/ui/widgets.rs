/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░░░██╗██╗░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░░██╔╝██║░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░██╔╝░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░███████║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚════██║██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░░░░░╚═╝╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use crate::engine::{CharState, TypingTest};
use crate::themes::Theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

// turn the test into styled lines: typed chars get colored by how you did,
// the next char gets the "cursor" treatment, the rest stay dim
pub fn build_test_lines(test: &TypingTest, theme: &Theme) -> Vec<Line<'static>> {
    let target = test.target_chars();
    let mut lines: Vec<Line> = Vec::new();
    let mut current = Vec::<Span>::new();
    let mut col = 0usize;
    let max_cols = 80usize;

    for (i, ch) in target.iter().enumerate() {
        let (style, display_char) = if i < test.typed.len() {
            let t = &test.typed[i];
            let style = match t.state {
                CharState::Correct => Style::default().fg(theme.correct()),
                CharState::Incorrect => Style::default()
                    .fg(theme.incorrect())
                    .add_modifier(Modifier::UNDERLINED),
                CharState::Extra => Style::default().fg(theme.incorrect()),
                CharState::Pending => Style::default().fg(theme.pending()),
            };
            (style, t.actual)
        } else if i == test.typed.len() {
            (
                Style::default()
                    .fg(theme.cursor())
                    .add_modifier(Modifier::REVERSED),
                *ch,
            )
        } else {
            (Style::default().fg(theme.pending()), *ch)
        };

        // wrap at word boundaries once we pass ~80 cols, no mid-word cuts
        if *ch == ' ' && col >= max_cols {
            lines.push(Line::from(std::mem::take(&mut current)));
            col = 0;
        }

        current.push(Span::styled(display_char.to_string(), style));
        let width = unicode_width::UnicodeWidthChar::width(display_char)
            .unwrap_or(1)
            .max(1);
        col += width;
    }

    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

// the little live stats strip in the footer
pub fn stat_spans(
    test: &TypingTest,
    theme: &Theme,
    show_wpm: bool,
    show_acc: bool,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    if show_wpm {
        spans.push(Span::styled(
            format!("{:>5.0} wpm", test.stats.wpm()),
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw("   "));
    }
    if show_acc {
        spans.push(Span::styled(
            format!("{:>4.1}% acc", test.stats.accuracy()),
            Style::default().fg(theme.accent()), // fixed
        ));
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("{:.1}s", test.stats.elapsed.as_secs_f64()),
        Style::default().fg(theme.sub()),
    ));
    spans
}

pub fn build_wpm_graph(test: &TypingTest, theme: &Theme, width: usize) -> Vec<Line<'static>>
{
    let samples = &test.stats.wpm_samples;

    if samples.is_empty() {
        return vec![Line::from(Span::styled(
            "wpm graph: waiting for data...",
            Style::default().fg(theme.sub()),
        ))];
    }

    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let max = samples.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
    let min = samples.iter().cloned().fold(f64::MAX, f64::min);
    let range = (max - min).max(1.0);

    let start = samples.len().saturating_sub(width);
    let visible = &samples[start..];

    let mut graph_spans = Vec::with_capacity(visible.len());
    for &s in visible {
        let normalized = (s - min) / range;
        let idx = ((normalized * (BLOCKS.len() - 1) as f64).round() as usize)
            .min(BLOCKS.len() - 1);
        graph_spans.push(Span::styled(
            BLOCKS[idx].to_string(),
            Style::default().fg(theme.accent()),
        ));
    }

    let label = format!(
        "wpm {:.0}-{:.0}  ",
        min,
        max
    );

    vec![
        Line::from(vec![
            Span::styled(label, Style::default().fg(theme.sub())),
            Span::raw(""),
        ]),
        Line::from(graph_spans),
    ]
}
