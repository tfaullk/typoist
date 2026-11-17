/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    Quit,
    RestartNewWords,
    ResetTest,
    ThemePicker,
    CycleWordSet,
    CycleMode,
    ToggleLiveWpm,
    ToggleAccuracy,
    ToggleGraph,
    OpenHelp,
    Back,
}

impl KeyAction {
    pub fn all() -> &'static [KeyAction] {
        &[
            KeyAction::RestartNewWords,
            KeyAction::ResetTest,
            KeyAction::ThemePicker,
            KeyAction::CycleWordSet,
            KeyAction::CycleMode,
            KeyAction::ToggleLiveWpm,
            KeyAction::ToggleAccuracy,
            KeyAction::ToggleGraph,
            KeyAction::OpenHelp,
            KeyAction::Back,
            KeyAction::Quit,
        ]
    }

    pub fn id(&self) -> &'static str {
        match self {
            KeyAction::Quit => "quit",
            KeyAction::RestartNewWords => "restart_new_words",
            KeyAction::ResetTest => "reset_test",
            KeyAction::ThemePicker => "theme_picker",
            KeyAction::CycleWordSet => "cycle_word_set",
            KeyAction::CycleMode => "cycle_mode",
            KeyAction::ToggleLiveWpm => "toggle_live_wpm",
            KeyAction::ToggleAccuracy => "toggle_accuracy",
            KeyAction::ToggleGraph => "toggle_graph",
            KeyAction::OpenHelp => "open_help",
            KeyAction::Back => "back",
        }
    }

    pub fn label(&self) -> &'static str {
        match self { 
            KeyAction::Quit => "Quit",
            KeyAction::RestartNewWords => "Restart (new words)",
            KeyAction::ResetTest => "Reset current test",
            KeyAction::ThemePicker => "Open theme picker",
            KeyAction::CycleWordSet => "Cycle word set",
            KeyAction::CycleMode => "Cycle mode",
            KeyAction::ToggleLiveWpm => "Toggle live WPM",
            KeyAction::ToggleAccuracy => "Toggle accuracy",
            KeyAction::ToggleGraph => "Toggle WPM graph",
            KeyAction::OpenHelp => "Open help",
            KeyAction::Back => "Back / quit when idle",
        }
    }

    pub fn default_spec(&self) -> KeySpec {
        match self {
            KeyAction::Quit => KeySpec::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyAction::RestartNewWords => KeySpec::new(KeyCode::Tab, KeyModifiers::NONE),
            KeyAction::ResetTest => KeySpec::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyAction::ThemePicker => KeySpec::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
            KeyAction::CycleWordSet => KeySpec::new(KeyCode::Char('w'), KeyModifiers::CONTROL),
            KeyAction::CycleMode => KeySpec::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
            KeyAction::ToggleLiveWpm => KeySpec::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
            KeyAction::ToggleAccuracy => KeySpec::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            KeyAction::ToggleGraph => KeySpec::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
            KeyAction::OpenHelp => KeySpec::new(KeyCode::Char('h'), KeyModifiers::CONTROL),
            KeyAction::Back => KeySpec::new(KeyCode::Esc, KeyModifiers::NONE),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

impl KeySpec {
    pub fn new(code: KeyCode, mods: KeyModifiers) -> Self {
        Self { code, mods }
    }

    pub fn parse(s: &str) -> Option<KeySpec> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        let mut mods = KeyModifiers::NONE;
        let mut rest = s.to_string();
        loop {
            let lower = rest.to_ascii_lowercase();
            if let Some(r) = lower.strip_prefix("ctrl+") {
                mods |= KeyModifiers::CONTROL;
                rest = rest[rest.len() - r.len()..].to_string();
            } else if let Some(r) = lower.strip_prefix("shift+") {
                mods |= KeyModifiers::SHIFT;
                rest = rest[rest.len() - r.len()..].to_string();
            } else if let Some(r) = lower.strip_prefix("alt+") {
                mods |= KeyModifiers::ALT;
                rest = rest[rest.len() - r.len()..].to_string();
            } else {
                break;
            }
        }
        let code = parse_code(&rest)?;
        Some(KeySpec { code, mods })
    }

    pub fn display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.mods.contains(KeyModifiers::CONTROL) {
            parts.push("ctrl".into());
        }
        if self.mods.contains(KeyModifiers::ALT) {
            parts.push("alt".into());
        }
        if self.mods.contains(KeyModifiers::SHIFT) {
            parts.push("shift".into());
        }
        parts.push(code_display(self.code));
        parts.join("+")
    }
    
    pub fn matches(&self, ev: &KeyEvent) -> bool {
        if ev.code != self.code {
            return false;
        }
        let relevant = KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT;
        (ev.modifiers & relevant) == (self.mods & relevant)
    }

    pub fn from_event(ev: &KeyEvent) -> KeySpec {
        KeySpec {
            code: ev.code,
            mods: ev.modifiers,
        }
    }
}

fn parse_code(s: &str) -> Option<KeyCode> {
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        "space" => Some(KeyCode::Char(' ')),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        _ => {
            if let Some(n) = lower.strip_prefix('f') {
                if let Ok(n) = n.parse::<u8>() {
                    if (1..=12).contains(&n) {
                        return Some(KeyCode::F(n));
                    }
                }
            }
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Some(KeyCode::Char(c)),
                _ => None,
            }
        }
    }
}

fn code_display(code: KeyCode) -> String {
    match code {
        KeyCode::Esc => "esc".into(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::Delete => "delete".into(),
        KeyCode::Insert => "insert".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Home => "home".into(),
        KeyCode::End => "end".into(),
        KeyCode::PageUp => "pageup".into(),
        KeyCode::PageDown => "pagedown".into(),
        KeyCode::F(n) => format!("f{}", n),
        KeyCode::Char(' ') => "space".into(),
        KeyCode::Char(c) => c.to_string(),
        _ => "?".into(),
    }
}

pub fn action_for(ev: &KeyEvent, settings: &crate::config::Settings) -> Option<KeyAction> {
    if settings.keybind_for(KeyAction::Quit).matches(ev) {
        return Some(KeyAction::Quit);
    }
    for action in KeyAction::all() {
        if *action == KeyAction::Quit {
            continue;
        }
        if settings.keybind_for(*action).matches(ev) {
            return Some(*action);
        }
    }
    None
}

pub fn conflict_for(
    spec: KeySpec,
    for_action: KeyAction,
    settings: &crate::config::Settings,
) -> Option<KeyAction> {
    for action in KeyAction::all() {
        if *action == for_action {
            continue;
        }
        if settings.keybind_for(*action) == spec {
            return Some(*action);
        }
    }
    None
}
