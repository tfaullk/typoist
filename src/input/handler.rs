/*



████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ░░███╗░░░░░██████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ░████║░░░░░╚════██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ██╔██║░░░░░░█████╔╝░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ╚═╝██║░░░░░░╚═══██╗░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗██████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝╚═════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk


*/

use crate::app::{App, Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub enum InputAction {
    Quit,
    None,
}

pub fn handle_key(app: &mut App, key: KeyEvent) -> InputAction {
    // error popup swallows everything until it's dismissed
    if app.current_theme_warning().is_some() {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Right | KeyCode::Down => {
                app.dismiss_theme_warning();
            }
            _ => {}
        }

        return InputAction::None;
    }

    let mods = key.modifiers;
    let only_ctrl = mods.contains(KeyModifiers::CONTROL)
        && !mods.contains(KeyModifiers::ALT)
        && !mods.contains(KeyModifiers::SHIFT);
    if only_ctrl && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('c')) {
        return InputAction::Quit;
    }
    
    match app.screen {
        Screen::Test => handle_test(app, key),
        Screen::Results => handle_results(app, key),
        Screen::ThemePicker => handle_theme_picker(app, key),
        // help closes on any keypress
        Screen::Help => {
            app.screen = Screen::Test;
        }
    }

    InputAction::None
}

fn handle_test(app: &mut App, key: KeyEvent) {
    let mods = key.modifiers;
    let ctrl = mods.contains(KeyModifiers::CONTROL)
        && !mods.contains(KeyModifiers::ALT);

    match key.code {
        // esc means "back" — mid-test it restarts with the same words, otherwise quit
        KeyCode::Esc => {
            use crate::engine::TestStatus;
            if app.test.status == TestStatus::Running {
                app.restart_test(false);
            } else {
                return_quit(app);
            }
        }
        KeyCode::Tab => app.restart_test(true),
        KeyCode::Backspace => app.test.backspace(),

        KeyCode::Char('t') if ctrl => {
            // jump straight to the current theme in the list
            app.screen = Screen::ThemePicker;
            app.theme_index = app
                .themes
                .iter()
                .position(|theme| theme.name == app.theme.name)
                .unwrap_or(0);
        }

        KeyCode::Char('w') if ctrl => app.cycle_word_set(),
        KeyCode::Char('n') if ctrl => app.cycle_mode(),
        KeyCode::Char('l') if ctrl => {
            app.settings.show_live_wpm = !app.settings.show_live_wpm;
        }
        KeyCode::Char('a') if ctrl => {
            app.settings.show_accuracy = !app.settings.show_accuracy;
        }
        KeyCode::Char('h') if ctrl => app.screen = Screen::Help,
        KeyCode::F(1) => app.screen = Screen::Help,

        KeyCode::Char(c) => {
            app.test.push_char(c);
            // finished mid-word means we ran out of text, show the results
            if app.test.status == crate::engine::TestStatus::Finished {
                app.screen = Screen::Results;
                let _ = app.settings.save();
            }
        }
        _ => {}
    }
}

fn handle_results(app: &mut App, key: KeyEvent) {
    let mods = key.modifiers;
    let ctrl = mods.contains(KeyModifiers::CONTROL)
        && !mods.contains(KeyModifiers::ALT);

    match key.code {
        KeyCode::Tab => {
            app.restart_test(true);
            app.screen = Screen::Test;
        }
        KeyCode::Esc => return_quit(app),
        KeyCode::Enter => {
            app.screen = Screen::Test;
            app.restart_test(true);
        }
        KeyCode::Char('t') if ctrl => app.screen = Screen::ThemePicker,
        KeyCode::Char('h') if ctrl => app.screen = Screen::Help,
        _ => {}
    }
}

fn handle_theme_picker(app: &mut App, key: KeyEvent) {
    let mods = key.modifiers;
    let ctrl = mods.contains(KeyModifiers::CONTROL)
        && !mods.contains(KeyModifiers::ALT);

    let theme_count = app.themes.len();

    if theme_count == 0 {
        return;
    }

    match key.code {
        KeyCode::Esc => app.screen = Screen::Test,
        KeyCode::Up => {
            // wrap around at the top
            if app.theme_index == 0 {
                app.theme_index = theme_count - 1;
            } else {
                app.theme_index -= 1;
            }

            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        KeyCode::Down => {
            app.theme_index = (app.theme_index + 1) % theme_count;
            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        // ctrl+p/ctrl+n as a vim-flavored alternative to the arrows
        KeyCode::Char('p') if ctrl => {
            if app.theme_index == 0 {
                app.theme_index = theme_count - 1;
            } else {
                app.theme_index -= 1;
            }

            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        KeyCode::Char('n') if ctrl => {
            app.theme_index = (app.theme_index + 1) % theme_count;
            let name = app.themes[app.theme_index].name.clone();
            app.preview_theme(&name);
        }
        // enter is the only thing that actually saves the pick
        KeyCode::Enter => {
            app.settings.theme = app.themes[app.theme_index].name.clone();
            let _ = app.settings.save();
            app.screen = Screen::Test;
        }
        _ => {}
    }
}

fn return_quit(app: &mut App) {
    app.should_quit = true;
}
