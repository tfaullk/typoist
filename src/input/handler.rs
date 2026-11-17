/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use crate::engine::TestStatus;
use crate::app::{App, Screen};
use crate::input::keybinds::{action_for, conflict_for, KeyAction, KeySpec};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum InputAction {
    Quit,
    None,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> InputAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('c')) {
        return InputAction::Quit;
    }

    if app.current_theme_warning().is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Right | KeyCode::Down => {
                app.dismiss_theme_warning();
            }
            _ => {}
        }
        return InputAction::None;
    }

    if app.keybind_capture.is_some() {
        capture_keybind(app, key);
        return InputAction::None;
    }

    match app.screen {
        Screen::Menu => handle_menu(app, key),
        Screen::Test => handle_test(app, key),
        Screen::Steno => handle_steno(app, key),
        Screen::Results => handle_results(app, key),
        Screen::ThemePicker => handle_theme_picker(app, key),
        Screen::Help => handle_help(app, key),
        Screen::Keybinds => handle_keybinds(app, key),
        Screen::Update => handle_update(app, key),
    }
}

fn dispatch(app: &mut App, action: KeyAction) -> InputAction {
    match action {
        KeyAction::Quit => return InputAction::Quit,
        KeyAction::RestartNewWords => {
            app.restart_test(true);
            app.screen = Screen::Test;
        }
        KeyAction::ResetTest => {
            if app.test.status == TestStatus::Running {
                app.restart_test(false);
            }
        }
        KeyAction::ThemePicker => {
            app.theme_index = app
                .themes
                .iter()
                .position(|t| t.name == app.theme.name)
                .unwrap_or(0);
            app.screen = Screen::ThemePicker;
        }
        KeyAction::CycleWordSet => {
            if app.screen == Screen::Test {
                app.cycle_word_set();
            }
        }
        KeyAction::CycleMode => {
            if app.screen == Screen::Test {
                app.cycle_mode();
            }
        }
        KeyAction::ToggleLiveWpm => {
            app.settings.show_live_wpm = !app.settings.show_live_wpm;
        }
        KeyAction::ToggleAccuracy => {
            app.settings.show_accuracy = !app.settings.show_accuracy;
        }
        KeyAction::ToggleGraph => {
            app.settings.show_wpm_graph = !app.settings.show_wpm_graph;
        }
        KeyAction::OpenHelp => {
            app.help_screen = app.screen;
            app.screen = Screen::Help;
        }
        KeyAction::Back => {
            app.screen = if app.test.status == TestStatus::NotStarted {
                Screen::Menu
            } else {
                Screen::Test
            };
        }
    }
    InputAction::None
}

fn handle_test(app: &mut App, key: KeyEvent) -> InputAction {
    if let Some(action) = action_for(&key, &app.settings) {
        if matches!(action, KeyAction::ResetTest | KeyAction::Back) {
            if app.test.status == TestStatus::Running {
                app.restart_test(false);
            } else {
                return InputAction::Quit;
            }
            return InputAction::None;
        }
        let res = dispatch(app, action);
        return res;
    }

    if key.code == KeyCode::Backspace && key.modifiers == KeyModifiers::NONE {
        app.test.backspace();
        return InputAction::None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) 
        || key.modifiers.contains(KeyModifiers::ALT)
    {
        return InputAction::None;
    }

    if let KeyCode::Char(c) = key.code {
        app.test.push_char(c);
        if app.test.status == TestStatus::Finished {
            app.finish_test_and_show_results();
        }
    }
    InputAction::None
}

fn handle_steno(app: &mut App, key: KeyEvent) -> InputAction {
    if let Some(action) = action_for(&key, &app.settings) {
        if matches!(action, KeyAction::ResetTest | KeyAction::Back) {
            if app.steno.status == crate::engine::StenoStatus::Running {
                app.restart_steno(false);
            } else {
                return InputAction::Quit;
            }
            return InputAction::None;
        }
        let res = dispatch(app, action);
        return res;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL)
        || key.modifiers.contains(KeyModifiers::ALT)
    {
        return InputAction::None;
    }

    match key.code {
        KeyCode::Char(c) => {
            if c == ' ' || c == '*' {
                app.steno.commit_chord();
            } else {
                app.steno.push_key(c);
            }
            if app.steno.status == crate::engine::StenoStatus::Finished {
                app.finish_steno_and_show_results();
            }
        }
        KeyCode::Enter => {
            app.steno.commit_chord();
            if app.steno.status == crate::engine::StenoStatus::Finished {
                app.finish_steno_and_show_results();
            }
        }
        KeyCode::Backspace => {
            app.steno.pending_keys.clear();
            app.steno.chord_deadline = None;
        }
        _ => {}
    }
    InputAction::None
}

fn handle_results(app: &mut App, key: KeyEvent) -> InputAction {
    if let Some(action) = action_for(&key, &app.settings) {
        match action {
            KeyAction::RestartNewWords | KeyAction::ResetTest => {
                app.restart_test(true);
                app.screen = Screen::Test;
                return InputAction::None;
            }
            KeyAction::Back => {
                app.screen = Screen::Menu;
                return InputAction::None;
            }
            KeyAction::ThemePicker => {
                app.screen = Screen::ThemePicker;
                return InputAction::None;
            }
            KeyAction::OpenHelp => {
                app.help_screen = Screen::Results;
                app.screen = Screen::Help;
                return InputAction::None;
            }
            _ => return InputAction::None,
        }
    }
    if key.code == KeyCode::Enter {
        app.restart_test(true);
        app.screen = Screen::Test;
    }
    InputAction::None
}

fn handle_theme_picker(app: &mut App, key: KeyEvent) -> InputAction {
    let theme_count = app.themes.len();
    if theme_count == 0 {
        return InputAction::None;
    }

    if let Some(action) = action_for(&key, &app.settings) {
        if matches!(action, KeyAction::Back | KeyAction::ResetTest) {
            app.screen = if app.test.status == TestStatus::NotStarted {
                Screen::Menu
            } else {
                Screen::Test
            };
            return InputAction::None;
        }
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.theme_index == 0 {
                app.theme_index = theme_count - 1;
            } else {
                app.theme_index -= 1;
            }
            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.theme_index = (app.theme_index + 1) % theme_count;
            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        KeyCode::Enter => {
            app.settings.theme = app.themes[app.theme_index].name.clone();
            let _ = app.settings.save();
            app.screen = Screen::Test;
        }
        _ => {}
    }
    InputAction::None
}

fn handle_menu(app: &mut App, key: KeyEvent) -> InputAction {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        KeyCode::Enter | KeyCode::Char(' ') => app.menu_activate(),
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
    InputAction::None
}

fn handle_help(app: &mut App, _key: KeyEvent) -> InputAction {
    app.screen = app.help_screen;
    InputAction::None
}

fn handle_update(app: &mut App, key: KeyEvent) -> InputAction {
    use crate::update::{tui_check, tui_install, TuiUpdateState};

    match (&app.update_state, key.code) {
        (_, KeyCode::Esc) => {
            app.update_state = TuiUpdateState::default();
            app.screen = Screen::Menu;
        }
        (TuiUpdateState::Idle, KeyCode::Enter) => {
            app.update_state = TuiUpdateState::Checking;
            app.update_state = tui_check();
        }
        (TuiUpdateState::Available { current, latest }, KeyCode::Enter) => {
            let (c, l ) = (current.clone(), latest.clone());
            app.update_state = TuiUpdateState::Installing;
            app.update_state = tui_install(c, l);
        }
        (TuiUpdateState::UpToDate { .. }, KeyCode::Enter) 
        | (TuiUpdateState::Done { .. }, KeyCode::Enter)
        | (TuiUpdateState::Error(_), KeyCode::Enter) => {
            app.update_state = TuiUpdateState::Idle;
        }
        _ => {}
    }
    InputAction::None
}

fn handle_keybinds(app: &mut App, key: KeyEvent) -> InputAction {
    let actions = KeyAction::all();
    let count = actions.len();
    if count == 0 {
        return InputAction::None;
    }

    if let Some(action) = action_for(&key, &app.settings) {
        if matches!(action, KeyAction::Back | KeyAction::ResetTest) {
            app.screen = Screen::Menu;
            app.keybind_message = None;
            return InputAction::None;
        } 
    }
    
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.keybind_index == 0 {
                app.keybind_index = count - 1;
            } else {
                app.keybind_index -= 1;
            }
            app.keybind_message = None;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.keybind_index = (app.keybind_index + 1) % count;
            app.keybind_message = None;
        }
        KeyCode::Enter => {
            app.keybind_capture = Some(actions[app.keybind_index]);
            app.keybind_message = None;
        }
        KeyCode::Delete | KeyCode::Backspace => {
            let action = actions[app.keybind_index];
            app.settings.set_keybind(action, None);
            app.keybind_message = Some(format!("Reset {} to default", action.label()));
            let _ = app.settings.save();
        }
        KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT) =>
        {
            app.settings.keybinds.clear();
            app.keybind_message = Some("All keybinds reset to defaults".into());
            let _ = app.settings.save();
        }
        _ => {}
    }
    InputAction::None
}

fn capture_keybind(app: &mut App, key: KeyEvent) {
    let Some(action) = app.keybind_capture else {
        return;
    };

    if key.code == KeyCode::Esc {
        app.keybind_capture = None;
        app.keybind_message = Some("Rebind cancelled".into());
        return;
    }

    if matches!(
        key.code,
        KeyCode::Null
            | KeyCode::CapsLock
            | KeyCode::ScrollLock
            | KeyCode::NumLock
            | KeyCode::PrintScreen
            | KeyCode::Pause
            | KeyCode::Menu
            | KeyCode::KeypadBegin
            | KeyCode::Media(_)
            | KeyCode::Modifier(_)
    ) {
        app.keybind_message = Some("Press a regular key (not a modifier alone)".into());
        return;
    }

    let spec = KeySpec::from_event(&key);

    if let Some(other) = conflict_for(spec, action, &app.settings) {
        app.keybind_message = Some(format!(
            "'{}' is already bound to '{}'",
            spec.display(),
            other.label()
        ));
        return;
    }

    let spec_str = spec.display();

    if action.default_spec() == spec {
        app.settings.set_keybind(action, None);
        app.keybind_message = Some(format!("'{}' restored to default ({})", action.label(), spec_str));
    } else {
        app.settings.set_keybind(action, Some(spec_str.clone()));
        app.keybind_message = Some(format!("'{}' bound to {}", action.label(), spec_str));
    }
    let _ = app.settings.save();
    app.keybind_capture = None;
}
