extern crate termion;

use std::io::{stdout, stdin, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct MenuItem {
    name: &'static str,
    sub_menu: Option<Vec<MenuItem>>,
    action: Option<fn()>,
}

fn generate_menu(menu: &[MenuItem], selected_option: usize) -> String {
    let mut output = String::new();
    for (i, item) in menu.iter().enumerate() {
        let mut title = item.name.to_string();
        if let Some(sub_menu) = item.sub_menu.as_ref() {
            if !sub_menu.is_empty() {
                title.push_str(" >");
            }
        }
        if i == selected_option {
            output.push_str(&format!("> \x1B[1m{} \x1B[0m<\r\n", title));
        } else {
            output.push_str(&format!("  \x1B[1m{} \x1B[0m  \r\n", title));
        }
    }
    output
}

fn display_menu(menu: &[MenuItem], selected_option: usize) {
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::clear::All).unwrap();
    write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
    let menu_output = generate_menu(menu, selected_option);
    write!(stdout, "{}", menu_output).unwrap();
    stdout.flush().unwrap();
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    let main_menu: Vec<MenuItem> = vec![
        MenuItem {
            name: "Option 1",
            sub_menu: Some(vec![
                MenuItem {
                    name: "Sub-option 1.1",
                    sub_menu: None,
                    action: Some(suboption1_1_function),
                },
                MenuItem {
                    name: "Sub-option 1.2",
                    sub_menu: None,
                    action: Some(suboption1_2_function),
                },
                MenuItem {
                    name: "Back",
                    sub_menu: None,
                    action: None,
                },
            ]),
            action: None,
        },
        MenuItem {
            name: "Option 2",
            sub_menu: Some(vec![
                MenuItem {
                    name: "Sub-option 2.1",
                    sub_menu: None,
                    action: Some(suboption2_1_function),
                },
                MenuItem {
                    name: "Sub-option 2.2",
                    sub_menu: None,
                    action: Some(suboption2_2_function),
                },
                MenuItem {
                    name: "Back",
                    sub_menu: None,
                    action: None,
                },
            ]),
            action: None,
        },
        MenuItem {
            name: "Option 3",
            sub_menu: None,
            action: Some(option3_function),
        },
        MenuItem {
            name: "Exit",
            sub_menu: None,
            action: None,
        },
    ];
    let mut selected_main_option = 0;
    let mut selected_sub_option = 0;
    let mut sub_menu_visible = false;

    write!(stdout, "{}", termion::clear::All).unwrap();
    stdout.flush().unwrap();

    loop {
        if sub_menu_visible {
            if let Some(sub_menu) = main_menu[selected_main_option].sub_menu.as_ref() {
                display_menu(sub_menu, selected_sub_option);
            } else {
                display_menu(&main_menu, selected_main_option);
            }
        } else {
            display_menu(&main_menu, selected_main_option);
        }

        let key = stdin.lock().keys().next().unwrap().unwrap();
        match key {
            Key::Up => {
                if sub_menu_visible {
                    if selected_sub_option > 0 {
                        selected_sub_option -= 1;
                    }
                } else {
                    if selected_main_option > 0 {
                        selected_main_option -= 1;
                    }
                }
            }
            Key::Down => {
                if sub_menu_visible {
                    if let Some(sub_menu) = main_menu[selected_main_option].sub_menu.as_ref() {
                        if selected_sub_option < sub_menu.len() - 1 {
                            selected_sub_option += 1;
                        }
                    }
                } else {
                    if selected_main_option < main_menu.len() - 1 {
                        selected_main_option += 1;
                    }
                }
            }
            Key::Char('\n') => {
                if sub_menu_visible {
                    if let Some(sub_menu) = main_menu[selected_main_option].sub_menu.as_ref() {
                        if selected_sub_option == sub_menu.len() - 1 {
                            sub_menu_visible = false;
                        } else if let Some(action) = sub_menu[selected_sub_option].action {
                            action();
                            wait_for_key();
                        }
                    }
                } else {
                    if let Some(sub_menu) = main_menu[selected_main_option].sub_menu.as_ref() {
                        if !sub_menu.is_empty() {
                            sub_menu_visible = true;
                            selected_sub_option = 0;
                        } else if let Some(action) = main_menu[selected_main_option].action {
                            action();
                            wait_for_key();
                        } else {
                            break;
                        }
                    } else if let Some(action) = main_menu[selected_main_option].action {
                        action();
                        wait_for_key();
                    } else {
                        break;
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

fn wait_for_key() {
    let stdin = stdin();
    let _ = stdin.lock().keys().next();
}

fn option3_function() {
    println!("Option 3 selected");
}

fn suboption1_1_function() {
    println!("Sub-option 1.1 selected");
}

fn suboption1_2_function() {
    println!("Sub-option 1.2 selected");
}

fn suboption2_1_function() {
    println!("Sub-option 2.1 selected");
}

fn suboption2_2_function() {
    println!("Sub-option 2.2 selected");
}
