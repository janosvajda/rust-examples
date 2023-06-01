extern crate termion;

use std::io::{stdout, stdin, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn generate_menu(items: &[&str], selected_option: usize) -> String {
    let mut menu = String::new();
    for (i, option) in items.iter().enumerate() {
        if i == selected_option {
            menu.push_str(&format!("> \x1B[1m{} \x1B[0m<\r\n", option));
        } else {
            menu.push_str(&format!("  {}  \r\n", option));
        }
    }
    menu
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let main_menu = vec!["Option 1", "Option 2", "Option 3", "Exit"];
    let mut selected_main_option = 0;
    let mut sub_menu_visible = false;
    let sub_menu = vec!["Sub-option 1", "Sub-option 2", "Sub-option 3", "Back"];

    write!(stdout, "{}", termion::clear::All).unwrap();
    stdout.flush().unwrap();

    loop {
        write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();

        if sub_menu_visible {
            let menu = generate_menu(&sub_menu, selected_main_option);
            write!(stdout, "{}", menu).unwrap();
        } else {
            let menu = generate_menu(&main_menu, selected_main_option);
            write!(stdout, "{}", menu).unwrap();
        }

        stdout.flush().unwrap();

        let key = stdin.lock().keys().next().unwrap().unwrap();
        match key {
            Key::Up => {
                if sub_menu_visible {
                    if selected_main_option > 0 {
                        selected_main_option -= 1;
                    }
                } else {
                    if selected_main_option > 0 {
                        selected_main_option -= 1;
                    }
                }
            }
            Key::Down => {
                if sub_menu_visible {
                    if selected_main_option < sub_menu.len() - 1 {
                        selected_main_option += 1;
                    }
                } else {
                    if selected_main_option < main_menu.len() - 1 {
                        selected_main_option += 1;
                    }
                }
            }
            Key::Char('\n') => {
                if sub_menu_visible {
                    if selected_main_option == sub_menu.len() - 1 {
                        sub_menu_visible = false;
                    }
                } else {
                    if selected_main_option == main_menu.len() - 1 {
                        break;
                    } else {
                        sub_menu_visible = true;
                        selected_main_option = 0;
                    }
                }
            }
            Key::Char('q') | Key::Ctrl('c') => break,
            _ => {}
        }
    }

    write!(stdout, "{}", termion::clear::All).unwrap();
    stdout.flush().unwrap();
}
