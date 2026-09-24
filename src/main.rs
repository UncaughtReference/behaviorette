use std::i64;

use color_eyre::eyre::{Ok as EyreOk, Result as EyreResult};
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind, KeyModifiers},
    prelude::*,
    widgets::{Block, Paragraph, List, BorderType, ListState, Padding},
    layout::{Constraint, Layout},
    style::{Stylize, Style},
    DefaultTerminal, Frame,
};

use ratatui_textarea::{TextArea, CursorMove};

use cli_clipboard::{ClipboardContext, ClipboardProvider};

struct AppState<'a> {
    block_focus:                    usize,
    decomp_text_area:               TextArea<'a>,
    binary_text_area:               TextArea<'a>,
    conversion_direction:           u8,
    current_method:                 String,
    help_list_state:                ListState,
    single_byte_address:            String,
    atimes4_text_area:              TextArea<'a>,
    object_flags:                   String,
    object_flags_text_area:         TextArea<'a>
}

macro_rules! split_to_vec {
    ($left:expr) => {
        match (&$left) {
            left_val => {
                left_val.clone().map(|s| s.to_string()).collect()
            }
        }
    }
}

fn main() -> EyreResult<()>
{
    let mut app_state = AppState {
        block_focus:                0,
        decomp_text_area:           TextArea::default(),
        binary_text_area:           TextArea::default(),
        conversion_direction:       0,
        current_method:             "".to_string(),
        help_list_state:            ListState::default(),
        single_byte_address:        "".to_string(),
        atimes4_text_area:          TextArea::default(),
        object_flags:               "".to_string(),
        object_flags_text_area:     TextArea::from(vec!["0000000000000000"]),
    };
    
    app_state.help_list_state.select_next(); 

    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = run(terminal, &mut app_state);

    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, mut app_state: &mut AppState) -> EyreResult<()> {
    loop {
        terminal.draw(|f| render(f, app_state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    event::KeyCode::Char('1') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            app_state.block_focus = 1;
                        }
                    },
                    event::KeyCode::Char('2') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            app_state.block_focus = 2;
                        }
                    },
                    event::KeyCode::Char('3') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            app_state.block_focus = 3;
                        }
                    },
                    event::KeyCode::Char('4') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            app_state.block_focus = 4;
                        }
                    },
                    event::KeyCode::Char('s') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            match app_state.conversion_direction {
                                0 => {
                                    app_state.conversion_direction = 1;
                                    convert_binary_to_decomp(&mut app_state);
                                },
                                1 => {
                                    app_state.conversion_direction = 0;
                                    convert_decomp_to_binary(&mut app_state);
                                },
                                _ => {},
                            }
                        }
                    },
                    event::KeyCode::Char('q') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            if app_state.block_focus == 1 && app_state.conversion_direction == 0 {
                                app_state.decomp_text_area = TextArea::from(vec![""]); 
                            } else if app_state.block_focus == 2 && app_state.conversion_direction == 1 {
                                app_state.binary_text_area = TextArea::from(vec![""]); 
                            } else if app_state.block_focus == 4 {
                                app_state.object_flags_text_area = TextArea::from(vec!["0000000000000000"]);
                            }
                        }
                    },
                    event::KeyCode::Char('p') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            if app_state.block_focus == 1 {
                                let mut clipboard = ClipboardContext::new().unwrap();
                                match clipboard.get_contents() {
                                    Ok(val) => {
                                        app_state.decomp_text_area = TextArea::from(vec![val]);
                                    },
                                    Err(_) => { },
                                }
                            } else if app_state.block_focus == 2 {
                                let mut clipboard = ClipboardContext::new().unwrap();
                                match clipboard.get_contents() {
                                    Ok(val) => {
                                        app_state.binary_text_area = TextArea::from(vec![val]);
                                    },
                                    Err(_) => { },
                                }
                            } 
                        }
                    },
                    event::KeyCode::Char('c') => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            if app_state.block_focus == 1 {
                                let mut clipboard = ClipboardContext::new().unwrap();
                                let mut decomp_content = String::new();
                                for line in app_state.decomp_text_area.lines() {
                                    decomp_content.push_str(line);
                                    decomp_content.push_str("\n");
                                }
                                let _ = clipboard.set_contents(decomp_content.to_owned());
                            } else if app_state.block_focus == 2 {
                                let mut clipboard = ClipboardContext::new().unwrap();
                                let mut binary_content = String::new();
                                for line in app_state.binary_text_area.lines() {
                                    binary_content.push_str(line);
                                    binary_content.push_str("\n");
                                }
                                let _ = clipboard.set_contents(binary_content.to_owned());
                            } else if app_state.block_focus == 4 {
                                let mut clipboard = ClipboardContext::new().unwrap();                                
                                let _ = clipboard.set_contents(calculate_flags(&app_state, true).to_owned());
                            }
                        }
                    },
                    event::KeyCode::PageUp => {
                        app_state.help_list_state.select(Some(0));
                    },
                    event::KeyCode::PageDown => {
                        app_state.help_list_state.select(Some(59));
                    },


                    event::KeyCode::Esc => {
                        app_state.block_focus = 0; 
                        if key.modifiers == KeyModifiers::SHIFT {
                            break; 
                        }
                    }
                    _ => {},
                }

            }
            if app_state.block_focus == 1 {
                if app_state.conversion_direction == 0 { 
                    if key.code != event::KeyCode::PageUp && key.code != event::KeyCode::PageDown { 
                        app_state.decomp_text_area.input(key); 
                        convert_decomp_to_binary(&mut app_state);
                    }
                }
                else {
                    if key.code == event::KeyCode::Up || key.code == event::KeyCode::Down || key.code == event::KeyCode::Left || key.code == event::KeyCode::Right {
                        app_state.decomp_text_area.input(key);
                    }
                }
            } else if app_state.block_focus == 2 {
                if app_state.conversion_direction == 1 {
                    if key.code != event::KeyCode::PageUp && key.code != event::KeyCode::PageDown { 
                        app_state.binary_text_area.input(key);
                        convert_binary_to_decomp(&mut app_state);
                        get_current_method(&mut app_state);
                    }
                } else {
                    if key.code == event::KeyCode::Up || key.code == event::KeyCode::Down || key.code == event::KeyCode::Left || key.code == event::KeyCode::Right {
                        app_state.binary_text_area.input(key);
                    }
                }
            } else if app_state.block_focus == 3 {
                if key.code != event::KeyCode::Enter && !(key.code == event::KeyCode::Char('m') && key.modifiers == KeyModifiers::CONTROL) {
                    app_state.atimes4_text_area.input(key);
                    update_atimes4(&mut app_state);
                }
            } else if app_state.block_focus == 4 {
                if key.code == event::KeyCode::Right && app_state.object_flags_text_area.cursor().1 < 15 {
                    app_state.object_flags_text_area.input(key);
                } else if key.code == event::KeyCode::Left {
                    app_state.object_flags_text_area.input(key);
                } else if key.code == event::KeyCode::Char('0') {
                    clear_flag(&mut app_state);
                } else if key.code == event::KeyCode::Char('1') {
                    set_flag(&mut app_state);
                }
            }
            if key.kind == KeyEventKind::Press {
                if key.modifiers == KeyModifiers::ALT {
                    if key.code == event::KeyCode::Up {
                        app_state.help_list_state.select_previous();
                    } else if key.code == event::KeyCode::Down {
                        if app_state.help_list_state.selected().unwrap() < 59 {
                            app_state.help_list_state.select_next();
                        }
                    }
                }
            }
        }
    }
    EyreOk(())
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    //-----------------------------------------------------------------
    let main_block = Block::bordered()
        .title(Line::from("behaviorette".white().bold()).centered())
        .title_bottom(Line::from(vec![
                "Escape Widget ".white().bold(),
                "<Esc> ".cyan().bold(),
                "Escape Program ".white().bold(),
                "<Shift+Esc> ".cyan().bold(),
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(0, &app_state))
        .padding(Padding::uniform(0));
    
    let main_block_inner_area = main_block.inner(frame.area());

    let main_layout = Layout::horizontal([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(70),
        Constraint::Percentage(30),
        ])
        .split(main_block_inner_area);
    //-----------------------------------------------------------------


    //-----------------------------------------------------------------
    let left_block = Block::default()
        .padding(Padding::uniform(0));

    let left_block_inner_area = left_block.inner(main_layout[0]);

    let left_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(70),
        Constraint::Percentage(30),
        ])
        .split(left_block_inner_area);
    //-----------------------------------------------------------------

   
    //-----------------------------------------------------------------
    let bhv_block = Block::default()
        .padding(Padding::uniform(0));

    let bhv_block_inner_area = bhv_block.inner(left_layout[0]);

    let bhv_layout = Layout::horizontal([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(50),
        Constraint::Percentage(50),
        ])
        .split(bhv_block_inner_area);
    //-----------------------------------------------------------------


    //-----------------------------------------------------------------
    let decomp_title: &str;
    match app_state.conversion_direction {
        0 => {
            decomp_title = "Decomp-esque BHV Script";
        },
        1 => {
            decomp_title = "Decomp-esque BHV Script (Read-only)";
        }
        _ => {
            decomp_title = "";
        }
    }
    

    let decomp_block = Block::bordered()
        .title(Line::from(decomp_title.white().bold()))
        .title_bottom(Line::from(vec![
                "Focus ".white().bold(),
                "<Ctrl+1> ".cyan().bold(),
                "Clear ".white().bold(),
                "<Ctrl+Q> ".cyan().bold(),
                "Copy ".white().bold(),
                "<Ctrl+C> ".cyan().bold(),
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(1, &app_state))
        .padding(Padding::uniform(0));
    
    let decomp_block_inner_area = decomp_block.inner(bhv_layout[0]);

    let decomp_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(100),
        ])
        .split(decomp_block_inner_area);
    //-----------------------------------------------------------------

    
    //-----------------------------------------------------------------
    let binary_title: &str;
    match app_state.conversion_direction {
        1 => {
            binary_title = "Binary BHV Script";
        },
        0 => {
            binary_title = "Binary BHV Script (Read-only)";
        }
        _ => {
            binary_title = "";
        }
    }

    let binary_block = Block::bordered()
        .title(Line::from(binary_title.white().bold()))
        .title_bottom(Line::from(vec![
                "Focus ".white().bold(),
                "<Ctrl+2> ".cyan().bold(),
                "Clear ".white().bold(),
                "<Ctrl+Q> ".cyan().bold(),
                "Copy ".white().bold(),
                "<Ctrl+C> ".cyan().bold(),        
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(2, &app_state))
        .padding(Padding::uniform(0));
    
    let binary_block_inner_area = binary_block.inner(bhv_layout[1]);

    let binary_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(100),
        ])
        .split(binary_block_inner_area);
    //-----------------------------------------------------------------


    //-----------------------------------------------------------------
    let help_block = Block::bordered()
        .title(Line::from("Help".white().bold()))
        .title_bottom(Line::from(vec![       
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(99, &app_state))
        .padding(Padding::uniform(0));
    
    let help_block_inner_area = help_block.inner(left_layout[1]);

    let help_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(100),
        ])
        .split(help_block_inner_area);
    //-----------------------------------------------------------------



    //-----------------------------------------------------------------
    let right_block = Block::default()
        .padding(Padding::uniform(0));

    let right_block_inner_area = right_block.inner(main_layout[1]);

    let right_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Percentage(65),
        Constraint::Length(3),
        Constraint::Fill(1),
        ])
        .split(right_block_inner_area);
    //-----------------------------------------------------------------
    

    //-----------------------------------------------------------------
    let help_list_block = Block::bordered()
        .title(Line::from("Help List".white().bold()))
        .title_bottom(Line::from(vec![ 
                "Scroll ".white().bold(),
                "<Alt+Up/Down> ".cyan().bold(),
                "Jump ".white().bold(),
                "<PageUp/PageDown> ".cyan().bold(),
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(99, &app_state))
        .padding(Padding::uniform(0));

    let help_list_block_inner_area = help_list_block.inner(right_layout[2]);

    let help_list_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Percentage(100),
        ])
        .split(help_list_block_inner_area);
    //-----------------------------------------------------------------
    

    //-----------------------------------------------------------------
    let atimes4_block = Block::bordered()
        .title(Line::from("Offset To Single Byte Address".white().bold()))
        .title_bottom(Line::from(vec![ 
                "Focus ".white().bold(),
                "<Ctrl+3> ".cyan().bold(),
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(3, &app_state))
        .padding(Padding::uniform(0));

    let atimes4_block_inner_area = atimes4_block.inner(right_layout[3]);

    let atimes4_layout = Layout::horizontal([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Length(4),
        Constraint::Fill(1),
        ])
        .split(atimes4_block_inner_area);
    //-----------------------------------------------------------------
    

    //-----------------------------------------------------------------
    let flags_block = Block::bordered()
        .title(Line::from("Object Flags Editor".white().bold()))
        .title_bottom(Line::from(vec![ 
                "Focus ".white().bold(),
                "<Ctrl+4> ".cyan().bold(),
                "Clear ".white().bold(),
                "<Ctrl+Q> ".cyan().bold(),
                "Copy ".white().bold(),
                "<Ctrl+C> ".cyan().bold(),
        ]))
        .border_type(BorderType::Rounded)
        .border_style(border_selection(4, &app_state))
        .padding(Padding::uniform(0));

    let flags_block_inner_area = flags_block.inner(right_layout[4]);

    let flags_layout = Layout::vertical([Constraint::Fill(1)])
        .margin(0)
        .constraints(vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        ])
        .split(flags_block_inner_area);
    //-----------------------------------------------------------------

    
    let conversion_direction_line;
    if app_state.conversion_direction == 0 {
        conversion_direction_line = Line::from(vec![
            "Conversion Direction: ".white().bold(),
            "Decomp-esque ".light_magenta().bold(),
            "-> ".white().bold(),
            "Binary ".light_yellow().bold(),
        ]);
    } else {
        conversion_direction_line = Line::from(vec![
            "Conversion Direction: ".white().bold(),
            "Binary ".light_yellow().bold(),
            "-> ".white().bold(),
            "Decomp-esque ".light_magenta().bold(),
        ]);
    }

    frame.render_widget(main_block, frame.area());
    frame.render_widget(left_block, main_layout[0]);
    frame.render_widget(bhv_block, left_layout[0]);
    frame.render_widget(decomp_block, bhv_layout[0]);
    frame.render_widget(binary_block, bhv_layout[1]);

    frame.render_widget(&app_state.decomp_text_area, decomp_layout[0]); 
    frame.render_widget(&app_state.binary_text_area, binary_layout[0]); 

    frame.render_widget(help_block, left_layout[1]);
    frame.render_widget(get_help_paragraph(&app_state), help_layout[0]);

    frame.render_widget(right_block, main_layout[1]);
    frame.render_widget(conversion_direction_line, right_layout[0]);
    frame.render_widget(Line::from(vec![
        "Switch Direction ".white().bold(),
        "<Ctrl+S>".cyan().bold(),
    ]), right_layout[1]);
    frame.render_widget(help_list_block, right_layout[2]);
    frame.render_stateful_widget(List::new(vec![
            "General".white(),
            "Current Command".white(),
            "00 BEGIN".white(),
            "01 DELAY".white(),
            "02 CALL".white(),
            "03 RETURN".white(),
            "04 GOTO".white(),
            "05 BEGIN_REPEAT".white(),
            "06 END_REPEAT".white(),
            "07 END_REPEAT_CONTINUE".white(),
            "08 BEGIN_LOOP".white(),
            "09 END_LOOP".white(),
            "0A BREAK".white(),
            "0B BREAK_UNUSED".white(),
            "0C CALL_NATIVE".white(),
            "0D ADD_FLOAT".white(),
            "0E SET_FLOAT".white(),
            "0F ADD_INT".white(),
            "10 SET_INT".white(),
            "11 OR_INT".white(),
            "12 BIT_CLEAR".white(),
            "13 SET_INT_RAND_RSHIFT".white(),
            "14 SET_RANDOM_FLOAT".white(),
            "15 SET_RANDOM_INT".white(),
            "16 ADD_RANDOM_FLOAT".white(),
            "17 ADD_INT_RAND_RSHIFT".white(),
            "18 CMD_NOP_1".white(),
            "19 CMD_NOP_2".white(),
            "1A CMD_NOP_3".white(),
            "1B SET_MODEL".white(),
            "1C SPAWN_CHILD".white(),
            "1D DEACTIVATE".white(),
            "1E DROP_TO_FLOOR".white(),
            "1F SUM_FLOAT".white(),
            "20 SUM_INT".white(),
            "21 BILLBOARD".white(),
            "22 HIDE".white(),
            "23 SET_HITBOX".white(),
            "24 CMD_NOP_4".white(),
            "25 DELAY_VAR".white(),
            "26 BEGIN_REPEAT_UNUSED".white(),
            "27 LOAD_ANIMATIONS".white(),
            "28 ANIMATE".white(),
            "29 SPAWN_CHILD_WITH_PARAM".white(),
            "2A LOAD_COLLISION_DATA".white(),
            "2B SET_HITBOX_WITH_OFFSET".white(),
            "2C SPAWN_OBJ".white(),
            "2D SET_HOME".white(),
            "2E SET_HURTBOX".white(),
            "2F SET_INTERACT_TYPE".white(),
            "30 SET_OBJ_PHYSICS".white(),
            "31 SET_INTERACT_SUBTYPE".white(),
            "32 SCALE".white(),
            "33 PARENT_BIT_CLEAR".white(),
            "34 ANIMATE_TEXTURE".white(),
            "35 DISABLE_RENDERING".white(),
            "36 SET_INT_UNUSED".white(),
            "37 SPAWN_WATER_DROPLET".white(),
            "Interaction Types 1".white(),
            "Interaction Types 2".white(),
    ])
        .highlight_style(Style::new().light_magenta().bold())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true), help_list_layout[0], &mut app_state.help_list_state);
    frame.render_widget(atimes4_block, right_layout[3]);
    frame.render_widget(&app_state.atimes4_text_area, atimes4_layout[0]);
    frame.render_widget(Line::from(app_state.single_byte_address.as_str().white().bold()), atimes4_layout[1]);
    frame.render_widget(flags_block, right_layout[4]);
    frame.render_widget(&app_state.object_flags_text_area, flags_layout[0]);
    frame.render_widget(Line::from(get_flag_name(&app_state).as_str().cyan().bold()), flags_layout[1]);
    frame.render_widget(Line::from(calculate_flags(&app_state, false).as_str().light_green().bold()), flags_layout[2]);
    frame.render_widget(Paragraph::new(vec![
            Line::from("Use the Left and Right arrow keys to move."),
            Line::from("Press 1 to set a flag, and 0 to unset it."),
    ]), flags_layout[3]);
}

fn border_selection(border_id: usize, app_state: &AppState) -> Style {
    if app_state.block_focus == border_id { Style::new().light_magenta() }
    else { Style::new().white() }
}

fn convert_decomp_to_binary(mut app_state: &mut AppState) {
    let mut binary_bhv: Vec<String> = Vec::new();
    let mut line_index: usize = 0;
    for line in app_state.decomp_text_area.lines() {
        if *line == "".to_string() {
            binary_bhv.push("".to_string());
            continue;
        }

        if !line.contains("(") || !line.contains(")") {
            if !line.contains("(") && !line.contains(")") {
                binary_bhv.push("Error converting line: missing parentheses".to_string()); 
            }
            else {
                binary_bhv.push("Error converting line: missing parenthese".to_string()); 
            }
            continue; 
        }

        let parenthese_split: Vec<String> = split_to_vec!(line.split("("));

        let argument = match parenthese_split.get(1) {
            Some(val) => {
                val.replace(")", "")
            },
            None => {
                binary_bhv.push("Error converting line: missing method".to_string());
                continue;
            }
        };

        let method = parenthese_split[0].as_str().to_uppercase();

        if line_index == app_state.decomp_text_area.cursor().0 {
            app_state.current_method = method.to_string();
        }
        
        let mut converted_line = "".to_string();

        if method == "BEGIN" {
            converted_line.push_str("00 ");
            let argument_uppercase = argument.to_uppercase();
            if argument_uppercase == "OBJ_LIST_GENACTOR" || argument_uppercase == "4" { 
                converted_line.push_str("04 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_SURFACE" || argument_uppercase == "9" {
                converted_line.push_str("09 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_DESTRUCTIVE" || argument_uppercase == "2" {
                converted_line.push_str("02 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_UNUSED_3" || argument_uppercase == "3"  {
                converted_line.push_str("03 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_PLAYER" || argument_uppercase == "0" {
                converted_line.push_str("00 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_PUSHABLE" || argument_uppercase == "5" {
                converted_line.push_str("05 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_LEVEL" || argument_uppercase == "6" {
                converted_line.push_str("06 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_UNUSED_7" || argument_uppercase == "7" {
                converted_line.push_str("07 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_DEFAULT" || argument_uppercase == "8" {
                converted_line.push_str("08 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_UNUSED_1" || argument_uppercase == "1" {
                converted_line.push_str("01 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_POLELIKE" || argument_uppercase == "10" {
                converted_line.push_str("0A 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_SPAWNER" || argument_uppercase == "11" {
                converted_line.push_str("0B 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else if argument_uppercase == "OBJ_LIST_UNIMPORTANT" || argument_uppercase == "12" {
                converted_line.push_str("0C 00 00");
                binary_bhv.push(converted_line);
                continue;
            } else {
                binary_bhv.push("Error converting line: invalid object list".to_string());
                continue;
            }
        } else if method == "BEGIN_LOOP" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("08 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "END_LOOP" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("09 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "BREAK" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("0A 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "BREAK_UNUSED" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("0B 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "RETURN" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("03 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "END_REPEAT" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("06 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "END_REPEAT_CONTINUE" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("07 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "END_REPEAT_CONTINUE" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("07 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "DEACTIVATE" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("1D 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "DROP_TO_FLOOR" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("1E 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "BILLBOARD" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("21 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "HIDE" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("22 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "SET_HOME" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("2D 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "DISABLE_RENDERING" {
            if argument != "" {
                binary_bhv.push("Error converting line: unnecessary argument".to_string());
                continue;
            } else {
                converted_line.push_str("35 00 00 00");
                binary_bhv.push(converted_line);
                continue;
            }
        } else if method == "DELAY" {
            match parse_two_byte_argument(&argument, "01 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "BEGIN_REPEAT" {
            match parse_two_byte_argument(&argument, "05 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "SCALE" {
            match parse_two_byte_argument(&argument, "32 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "CMD_NOP_1" {
            match parse_two_byte_argument(&argument, "18 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "CMD_NOP_2" {
            match parse_two_byte_argument(&argument, "19 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "CMD_NOP_3" {
            match parse_two_byte_argument(&argument, "1A 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "SET_MODEL" {
            match parse_two_byte_argument(&argument, "1B 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "CALL" {
            match parse_four_byte_argument(&argument, "02 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "GOTO" {
            match parse_four_byte_argument(&argument, "04 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "CALL_NATIVE" {
            match parse_four_byte_argument(&argument, "0C 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "LOAD_COLLISION_DATA" {
            match parse_four_byte_argument(&argument, "2A 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "SET_INTERACT_TYPE" {
            match parse_four_byte_argument(&argument, "2F 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "SET_INTERACT_SUBTYPE" {
            match parse_four_byte_argument(&argument, "31 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "SPAWN_WATER_DROPLET" {
            match parse_four_byte_argument(&argument, "37 00 00 00 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    binary_bhv.push(val.to_string());
                }
            } 
        } else if method == "ADD_FLOAT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("0D ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "SET_FLOAT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("0E ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "ADD_INT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("0F ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "SET_INT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("10 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "OR_INT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("11 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "BIT_CLEAR" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("12 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "CMD_NOP_4" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("24 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "ANIMATE_TEXTURE" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("34 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "DELAY_VAR" {
            match parse_one_byte_argument(&argument, "25 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    converted_line.push_str(&val);
                    converted_line.push_str(" 00 00");
                    binary_bhv.push(converted_line);
                }
            } 
        } else if method == "BEGIN_REPEAT_UNUSED" {
            match parse_one_byte_argument(&argument, "26 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    converted_line.push_str(&val);
                    converted_line.push_str(" 00 00");
                    binary_bhv.push(converted_line);
                }
            } 
        } else if method == "ANIMATE" {
            match parse_one_byte_argument(&argument, "28 ") {
                Err(err_msg) => {
                    binary_bhv.push(err_msg);
                    continue;
                },
                Ok(val) => {
                    converted_line.push_str(&val);
                    converted_line.push_str(" 00 00");
                    binary_bhv.push(converted_line);
                }
            } 
        } else if method == "SET_INT_RAND_RSHIFT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("13 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SET_RANDOM_FLOAT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("14 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SET_RANDOM_INT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("15 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "ADD_RANDOM_FLOAT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("16 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "ADD_INT_RAND_RSHIFT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("17 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SPAWN_CHILD" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_four_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("1C 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_four_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "SPAWN_OBJ" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_four_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("2C 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_four_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }  
        } else if method == "SUM_FLOAT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("1F ");
                            converted_line.push_str(&val);
                            match parse_one_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_one_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SUM_INT" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("20 ");
                            converted_line.push_str(&val);
                            match parse_one_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_one_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SET_HITBOX" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_two_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("23 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }
        } else if method == "SET_HURTBOX" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_two_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("2E 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }
        } else if method == "LOAD_ANIMATIONS" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("27 ");
                            converted_line.push_str(&val);
                            match parse_four_byte_argument(&arguments[1], " 00 00 ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }
        } else if method == "PARENT_BIT_CLEAR" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("33 ");
                            converted_line.push_str(&val);
                            match parse_four_byte_argument(&arguments[1], " 00 00 ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }
        } else if method == "SPAWN_CHILD_WITH_PARAM" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_two_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("29 00 ");
                            converted_line.push_str(&val);
                            match parse_four_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_four_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SET_HITBOX_WITH_OFFSET" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 3 {
                    match parse_two_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("2B 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            converted_line.push_str(" 00 00");
                                            binary_bhv.push(converted_line);
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 3 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 3 arguments required".to_string());
                continue;
            } 
        } else if method == "SET_INT_UNUSED" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 2 {
                    match parse_one_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("36 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " 00 00 ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    converted_line.push_str(" 00 00");
                                    binary_bhv.push(converted_line);
                                }
                            }

                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 2 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 2 arguments required".to_string());
                continue;
            }
        } else if method == "SET_OBJ_PHYSICS" {
            if argument.contains(",") {
                let arguments: Vec<String> = split_to_vec!(argument.split(", "));
                if arguments.len() == 8 {
                    match parse_two_byte_argument(&arguments[0], "") {
                        Err(err_msg) => {
                            binary_bhv.push(err_msg);
                            continue;
                        },
                        Ok(val) => {
                            converted_line.push_str("30 00 00 00 ");
                            converted_line.push_str(&val);
                            match parse_two_byte_argument(&arguments[1], " ") {
                                Err(err_msg) => {
                                    binary_bhv.push(err_msg);
                                    continue;
                                },
                                Ok(val) => {
                                    converted_line.push_str(&val);
                                    match parse_two_byte_argument(&arguments[2], " ") {
                                        Err(err_msg) => {
                                            binary_bhv.push(err_msg);
                                            continue;
                                        },
                                        Ok(val) => {
                                            converted_line.push_str(&val);
                                            match parse_two_byte_argument(&arguments[3], " ") {
                                                Err(err_msg) => {
                                                    binary_bhv.push(err_msg);
                                                    continue;
                                                },
                                                Ok(val) => {
                                                    converted_line.push_str(&val);
                                                    match parse_two_byte_argument(&arguments[4], " ") {
                                                        Err(err_msg) => {
                                                            binary_bhv.push(err_msg);
                                                            continue;
                                                        },
                                                        Ok(val) => {
                                                            converted_line.push_str(&val);
                                                            match parse_two_byte_argument(&arguments[5], " ") {
                                                                Err(err_msg) => {
                                                                    binary_bhv.push(err_msg);
                                                                    continue;
                                                                },
                                                                Ok(val) => {
                                                                    converted_line.push_str(&val);
                                                                    match parse_two_byte_argument(&arguments[6], " ") {
                                                                        Err(err_msg) => {
                                                                            binary_bhv.push(err_msg);
                                                                            continue;
                                                                        },
                                                                        Ok(val) => {
                                                                            converted_line.push_str(&val);
                                                                            match parse_two_byte_argument(&arguments[7], " ") {
                                                                                Err(err_msg) => {
                                                                                    binary_bhv.push(err_msg);
                                                                                    continue;
                                                                                },
                                                                                Ok(val) => {
                                                                                    converted_line.push_str(&val);
                                                                                    binary_bhv.push(converted_line);
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                } else {
                    binary_bhv.push("Error converting line: 8 arguments required".to_string());
                    continue;
                } 
            } else {
                binary_bhv.push("Error converting line: 8 arguments required".to_string());
                continue;
            } 
        } else {
            binary_bhv.push("Error converting line: unknown command".to_string());
            continue;
        }
        line_index += 1;
    }
    app_state.binary_text_area = TextArea::from(binary_bhv);
}

fn parse_two_byte_argument(argument: &str, opcode: &str) -> Result<String, String> {
    if argument == "" {
        return Err("Error converting line: no given argument".to_string());
    }

    let mut converted_line = "".to_string();
    let mut can_check_second_character = true;
    match argument.chars().nth(1) {
        Some(_) => { },
        None => { can_check_second_character = false; },
    }
    if can_check_second_character {
        if argument.chars().nth(1).unwrap() == 'x' && argument.chars().next().unwrap() == '0' {
            match argument.chars().nth(2) {
                Some(_) => { },
                None => { 
                    return Err("Error converting line: no value found after '0x'".to_string());
                }
            }
            converted_line.push_str(opcode);
            let mut final_binary_hex_value = "".to_string();
            let mut hex_value_split: Vec<String> = split_to_vec!(argument.split("x"));
            hex_value_split = vec![hex_value_split[0].clone(), hex_value_split[1].replace(",", "")];

            match i64::from_str_radix(&hex_value_split[1], 16) {
                Ok(_) => { },
                Err(_) => {
                    return Err("Error converting line: invalid hex value".to_string());
                },
            }

            let char_count = hex_value_split[1].chars().count();
            match char_count {
                1 => {
                    final_binary_hex_value.push_str("00 0");
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                2 => {
                    final_binary_hex_value.push_str("00 ");
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                3 => {
                    final_binary_hex_value.push_str("0");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                },
                4 => {
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                },
                _ => {
                    return Err("Error converting line: 2-byte value required".to_string());
                }
            };
            converted_line.push_str(&final_binary_hex_value);
        } else {
            return Err("Error converting line: prefix hex values with '0x'".to_string());
        }
    } else {
        return Err("Error converting line: prefix hex values with '0x'".to_string());
    }
    Ok(converted_line)
}

fn parse_one_byte_argument(argument: &str, opcode: &str) -> Result<String, String> {
    if argument == "" {
        return Err("Error converting line: no given argument".to_string());
    }

    let mut converted_line = "".to_string();
    let mut can_check_second_character = true;
    match argument.chars().nth(1) {
        Some(_) => { },
        None => { can_check_second_character = false; },
    }
    if can_check_second_character {
        if argument.chars().nth(1).unwrap() == 'x' && argument.chars().next().unwrap() == '0' {
            match argument.chars().nth(2) {
                Some(_) => { },
                None => { 
                    return Err("Error converting line: no value found after '0x'".to_string());
                }
            }
            converted_line.push_str(opcode);
            let mut final_binary_hex_value = "".to_string();
            let mut hex_value_split: Vec<String> = split_to_vec!(argument.split("x"));
            hex_value_split = vec![hex_value_split[0].clone(), hex_value_split[1].replace(",", "")];

            match i64::from_str_radix(&hex_value_split[1], 16) {
                Ok(_) => { },
                Err(_) => {
                    return Err("Error converting line: invalid hex value".to_string());
                },
            }

            let char_count = hex_value_split[1].chars().count();
            match char_count {
                1 => {
                    final_binary_hex_value.push_str("0");
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                2 => {
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                _ => {
                    return Err("Error converting line: 1-byte value required".to_string());
                }
            };
            converted_line.push_str(&final_binary_hex_value);
        } else {
            return Err("Error converting line: prefix hex values with '0x'".to_string());
        }
    } else {
        return Err("Error converting line: prefix hex values with '0x'".to_string());
    }
    Ok(converted_line)
}

fn parse_four_byte_argument(argument: &str, opcode: &str) -> Result<String, String> {
    if argument == "" {
        return Err("Error converting line: no given argument".to_string());
    }

    let mut converted_line = "".to_string();
    let mut can_check_second_character = true;
    match argument.chars().nth(1) {
        Some(_) => { },
        None => { can_check_second_character = false; },
    }
    if can_check_second_character {
        if argument.chars().nth(1).unwrap() == 'x' && argument.chars().next().unwrap() == '0' {
            match argument.chars().nth(2) {
                Some(_) => { },
                None => { 
                    return Err("Error converting line: no value found after '0x'".to_string());
                }
            }
            converted_line.push_str(opcode);
            let mut final_binary_hex_value = "".to_string();
            let mut hex_value_split: Vec<String> = split_to_vec!(argument.split("x"));
            hex_value_split = vec![hex_value_split[0].clone(), hex_value_split[1].replace(",", "")];

            match i64::from_str_radix(&hex_value_split[1], 16) {
                Ok(_) => { },
                Err(_) => {
                    return Err("Error converting line: invalid hex value".to_string());
                },
            }

            let char_count = hex_value_split[1].chars().count();
            match char_count {
                1 => {
                    final_binary_hex_value.push_str("00 00 00 0");
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                2 => {
                    final_binary_hex_value.push_str("00 00 00 ");
                    final_binary_hex_value.push_str(&hex_value_split[1]);
                },
                3 => {
                    final_binary_hex_value.push_str("00 00 0");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                },
                4 => {
                    final_binary_hex_value.push_str("00 00 ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                },
                5 => {
                    final_binary_hex_value.push_str("00 0");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(4).unwrap().to_string());
                },
                6 => {
                    final_binary_hex_value.push_str("00 ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(4).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(5).unwrap().to_string());
                },
                7 => {
                    final_binary_hex_value.push_str("0");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(4).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(5).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(6).unwrap().to_string());
                },
                8 => {
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().next().unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(1).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(2).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(3).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(4).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(5).unwrap().to_string());
                    final_binary_hex_value.push_str(" ");
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(6).unwrap().to_string());
                    final_binary_hex_value.push_str(&hex_value_split[1].chars().nth(7).unwrap().to_string());
                },
                _ => {
                    return Err("Error converting line: 4-byte value required".to_string());
                }
            };
            converted_line.push_str(&final_binary_hex_value);
        } else {
            return Err("Error converting line: prefix hex values with '0x'".to_string());
        }
    } else {
        return Err("Error converting line: prefix hex values with '0x'".to_string());
    }
    Ok(converted_line)
}

fn convert_binary_to_decomp(mut app_state: &mut AppState) {
    let mut decomp_bhv: Vec<String> = Vec::new();
    for line in app_state.binary_text_area.lines() {
        if *line == "".to_string() {
            decomp_bhv.push("".to_string());
            continue;
        }

        let mut line_space_split: Vec<String> = split_to_vec!(line.to_uppercase().split(" "));
        let mut converted_line = "".to_string();

        for (i, line_space_splitee) in line_space_split.clone().into_iter().enumerate() {
            if line_space_splitee == " ".to_string() {
                line_space_split.remove(i);
            } else if line_space_splitee.chars().count() != 2 {
                line_space_split.remove(i);
            } 
        }

        if line_space_split.len() == 0 {
            decomp_bhv.push("Error converting line: invalid command".to_string());
            continue;
        }
        
        if line_space_split[0] == "00" {
            if line_space_split.len() == 4 {
                converted_line.push_str("BEGIN(");
                if line_space_split[1] == "00" {
                    converted_line.push_str("OBJ_LIST_PLAYER");
                } else if line_space_split[1] == "01" {
                    converted_line.push_str("OBJ_LIST_UNUSED_1");
                } else if line_space_split[1] == "02" {
                    converted_line.push_str("OBJ_LIST_DESTRUCTIVE");
                } else if line_space_split[1] == "03" {
                    converted_line.push_str("OBJ_LIST_UNUSED_3");
                } else if line_space_split[1] == "04" {
                    converted_line.push_str("OBJ_LIST_GENACTOR");
                } else if line_space_split[1] == "05" {
                    converted_line.push_str("OBJ_LIST_PUSHABLE");
                } else if line_space_split[1] == "06" {
                    converted_line.push_str("OBJ_LIST_LEVEL");
                } else if line_space_split[1] == "07" {
                    converted_line.push_str("OBJ_LIST_UNUSED_7");
                } else if line_space_split[1] == "08" {
                    converted_line.push_str("OBJ_LIST_DEFAULT");
                } else if line_space_split[1] == "09" {
                    converted_line.push_str("OBJ_LIST_SURFACE");
                } else if line_space_split[1] == "0A" {
                    converted_line.push_str("OBJ_LIST_POLELIKE");
                } else if line_space_split[1] == "0B" {
                    converted_line.push_str("OBJ_LIST_SPAWNER");
                } else if line_space_split[1] == "0C" {
                    converted_line.push_str("OBJ_LIST_UNIMPORTANT");
                } else {
                    decomp_bhv.push("Error converting line: invalid 2nd byte".to_string());
                    continue;
                }
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "01" {
            if line_space_split.len() == 4 {
                converted_line.push_str("DELAY(0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "05" {
            if line_space_split.len() == 4 {
                converted_line.push_str("BEGIN_REPEAT(0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "1B" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SET_MODEL(0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "32" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SCALE(0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "03" {
            converted_line.push_str("RETURN()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "06" {
            converted_line.push_str("END_REPEAT()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "07" {
            converted_line.push_str("END_REPEAT_CONTINUE()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "08" {
            converted_line.push_str("BEGIN_LOOP()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "09" {
            converted_line.push_str("END_LOOP()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "0A" {
            converted_line.push_str("BREAK()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "0B" {
            converted_line.push_str("BREAK_UNUSED()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "18" {
            converted_line.push_str("CMD_NOP_1()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "19" {
            converted_line.push_str("CMD_NOP_2()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "1A" {
            converted_line.push_str("CMD_NOP_3()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "1D" {
            converted_line.push_str("DEACTIVATE()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "1E" {
            converted_line.push_str("DROP_TO_FLOOR()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "21" {
            converted_line.push_str("BILLBOARD()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "22" {
            converted_line.push_str("HIDE()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "2D" {
            converted_line.push_str("SET_HOME()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "35" {
            converted_line.push_str("DISABLE_RENDERING()");
            decomp_bhv.push(converted_line);
        } else if line_space_split[0] == "02" {
            if line_space_split.len() == 8 {
                converted_line.push_str("CALL(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "04" {
            if line_space_split.len() == 8 {
                converted_line.push_str("GOTO(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "0C" {
            if line_space_split.len() == 8 {
                converted_line.push_str("CALL_NATIVE(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "2A" {
            if line_space_split.len() == 8 {
                converted_line.push_str("LOAD_COLLISION_DATA(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "2F" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_INTERACT_TYPE(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "31" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_INTERACT_SUBTYPE(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "37" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SPAWN_WATER_DROPLET(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "0D" {
            if line_space_split.len() == 4 {
                converted_line.push_str("ADD_FLOAT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "0E" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SET_FLOAT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "0F" {
            if line_space_split.len() == 4 {
                converted_line.push_str("ADD_INT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "10" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SET_INT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "11" {
            if line_space_split.len() == 4 {
                converted_line.push_str("OR_INT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "12" {
            if line_space_split.len() == 4 {
                converted_line.push_str("BIT_CLEAR(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "24" {
            if line_space_split.len() == 4 {
                converted_line.push_str("CMD_NOP_4(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "34" {
            if line_space_split.len() == 4 {
                converted_line.push_str("ANIMATE_TEXTURE(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "13" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_INT_RAND_RSHIFT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "14" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_RANDOM_FLOAT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "15" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_RANDOM_INT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "16" {
            if line_space_split.len() == 8 {
                converted_line.push_str("ADD_RANDOM_FLOAT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "17" {
            if line_space_split.len() == 8 {
                converted_line.push_str("ADD_INT_RAND_RSHIFT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "1C" {
            if line_space_split.len() == 12 {
                converted_line.push_str("SPAWN_CHILD(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[8]);
                converted_line.push_str(&line_space_split[9]);
                converted_line.push_str(&line_space_split[10]);
                converted_line.push_str(&line_space_split[11]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 12 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "2C" {
            if line_space_split.len() == 12 {
                converted_line.push_str("SPAWN_OBJ(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[8]);
                converted_line.push_str(&line_space_split[9]);
                converted_line.push_str(&line_space_split[10]);
                converted_line.push_str(&line_space_split[11]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 12 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "1F" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SUM_FLOAT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "20" {
            if line_space_split.len() == 4 {
                converted_line.push_str("SUM_INT(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "23" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_HITBOX(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "2E" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_HURTBOX(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "25" {
            if line_space_split.len() == 4 {
                converted_line.push_str("DELAY_VAR(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "26" {
            if line_space_split.len() == 4 {
                converted_line.push_str("BEGIN_REPEAT_UNUSED(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "28" {
            if line_space_split.len() == 4 {
                converted_line.push_str("ANIMATE(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 4 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "27" {
            if line_space_split.len() == 8 {
                converted_line.push_str("LOAD_ANIMATIONS(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "33" {
            if line_space_split.len() == 8 {
                converted_line.push_str("PARENT_BIT_CLEAR(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "29" {
            if line_space_split.len() == 12 {
                converted_line.push_str("SPAWN_CHILD_WITH_PARAM(0x");
                converted_line.push_str(&line_space_split[2]);
                converted_line.push_str(&line_space_split[3]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[8]);
                converted_line.push_str(&line_space_split[9]);
                converted_line.push_str(&line_space_split[10]);
                converted_line.push_str(&line_space_split[11]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 12 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "2B" {
            if line_space_split.len() == 12 {
                converted_line.push_str("SET_HITBOX_WITH_OFFSET(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[8]);
                converted_line.push_str(&line_space_split[9]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 12 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "36" {
            if line_space_split.len() == 8 {
                converted_line.push_str("SET_INT_UNUSED(0x");
                converted_line.push_str(&line_space_split[1]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 8 bytes required".to_string());
                continue;
            }
        } else if line_space_split[0] == "30" {
            if line_space_split.len() == 20 {
                converted_line.push_str("SET_OBJ_PHYSICS(0x");
                converted_line.push_str(&line_space_split[4]);
                converted_line.push_str(&line_space_split[5]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[6]);
                converted_line.push_str(&line_space_split[7]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[8]);
                converted_line.push_str(&line_space_split[9]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[10]);
                converted_line.push_str(&line_space_split[11]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[12]);
                converted_line.push_str(&line_space_split[13]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[14]);
                converted_line.push_str(&line_space_split[15]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[16]);
                converted_line.push_str(&line_space_split[17]);
                converted_line.push_str(", 0x");
                converted_line.push_str(&line_space_split[18]);
                converted_line.push_str(&line_space_split[19]);
                converted_line.push_str(")");
                decomp_bhv.push(converted_line);
            } else {
                decomp_bhv.push("Error converting line: length of 20 bytes required".to_string());
                continue;
            }
        } else {
            decomp_bhv.push("Error converting line: unknown command".to_string());
            continue;
        }
    }
    app_state.decomp_text_area = TextArea::from(decomp_bhv);
}

fn get_help_paragraph<'a>(app_state: &'a AppState<'a>) -> Paragraph<'a> {
    let mut current_method_str = "";
    let current_selection = app_state.help_list_state.selected().unwrap();
    if current_selection == 0 {
        return Paragraph::new(vec![
            Line::from("Hello there!".light_cyan()),
            Line::from("This program aims to make writing and reading behavior scripts simpler and easier."),
            Line::from("Read the README.md for more information.".white()),
            Line::from("Program by UncaughtReference".light_magenta().bold()),
            Line::from("Testers: ZennyTheZtarBones, lanayxy, ThatAussieGhost".light_yellow().bold()),
            Line::from("Help descriptions taken from n64decomp/sm64 and the hack64 wiki.".light_green().bold()),
            Line::from("Have fun converting your behavior scripts!".white().bold()),
        ]); 
    } else if current_selection == 1 {
        current_method_str = app_state.current_method.as_str();
    } else if current_selection == 2 {
        current_method_str = "BEGIN"; 
    } else if current_selection == 3 {
        current_method_str = "DELAY"; 
    } else if current_selection == 4 {
        current_method_str = "CALL"; 
    } else if current_selection == 5 {
        current_method_str = "RETURN"; 
    } else if current_selection == 6 {
        current_method_str = "GOTO"; 
    } else if current_selection == 7 {
        current_method_str = "BEGIN_REPEAT"; 
    } else if current_selection == 8 {
        current_method_str = "END_REPEAT"; 
    } else if current_selection == 9 {
        current_method_str = "END_REPEAT_CONTINUE"; 
    } else if current_selection == 10 {
        current_method_str = "BEGIN_LOOP"; 
    } else if current_selection == 11 {
        current_method_str = "END_LOOP"; 
    } else if current_selection == 12 {
        current_method_str = "BREAK"; 
    } else if current_selection == 13 {
        current_method_str = "BREAK_UNUSED"; 
    } else if current_selection == 14 {
        current_method_str = "CALL_NATIVE"; 
    } else if current_selection == 15 {
        current_method_str = "ADD_FLOAT"; 
    } else if current_selection == 16 {
        current_method_str = "SET_FLOAT"; 
    } else if current_selection == 17 {
        current_method_str = "ADD_INT"; 
    } else if current_selection == 18 {
        current_method_str = "SET_INT"; 
    } else if current_selection == 19 {
        current_method_str = "OR_INT"; 
    } else if current_selection == 20 {
        current_method_str = "BIT_CLEAR"; 
    } else if current_selection == 21 {
        current_method_str = "SET_INT_RAND_RSHIFT"; 
    } else if current_selection == 22 {
        current_method_str = "SET_RANDOM_FLOAT"; 
    } else if current_selection == 23 {
        current_method_str = "SET_RANDOM_INT"; 
    } else if current_selection == 24 {
        current_method_str = "ADD_RANDOM_FLOAT"; 
    } else if current_selection == 25 {
        current_method_str = "ADD_INT_RAND_RSHIFT"; 
    } else if current_selection == 26 {
        current_method_str = "CMD_NOP_1"; 
    } else if current_selection == 27 {
        current_method_str = "CMD_NOP_2"; 
    } else if current_selection == 28 {
        current_method_str = "CMD_NOP_3"; 
    } else if current_selection == 29 {
        current_method_str = "SET_MODEL"; 
    } else if current_selection == 30 {
        current_method_str = "SPAWN_CHILD"; 
    } else if current_selection == 31 {
        current_method_str = "DEACTIVATE"; 
    } else if current_selection == 32 {
        current_method_str = "DROP_TO_FLOOR"; 
    } else if current_selection == 33 {
        current_method_str = "SUM_FLOAT"; 
    } else if current_selection == 34 {
        current_method_str = "SUM_INT"; 
    } else if current_selection == 35 {
        current_method_str = "BILLBOARD"; 
    } else if current_selection == 36 {
        current_method_str = "HIDE"; 
    } else if current_selection == 37 {
        current_method_str = "SET_HITBOX"; 
    } else if current_selection == 38 {
        current_method_str = "CMD_NOP_4"; 
    } else if current_selection == 39 {
        current_method_str = "DELAY_VAR"; 
    } else if current_selection == 40 {
        current_method_str = "BEGIN_REPEAT_UNUSED"; 
    } else if current_selection == 41 {
        current_method_str = "LOAD_ANIMATIONS"; 
    } else if current_selection == 42 {
        current_method_str = "ANIMATE"; 
    } else if current_selection == 43 {
        current_method_str = "SPAWN_CHILD_WITH_PARAM"; 
    } else if current_selection == 44 {
        current_method_str = "LOAD_COLLISION_DATA"; 
    } else if current_selection == 45 {
        current_method_str = "SET_HITBOX_WITH_OFFSET"; 
    } else if current_selection == 46 {
        current_method_str = "SPAWN_OBJ"; 
    } else if current_selection == 47 {
        current_method_str = "SET_HOME"; 
    } else if current_selection == 48 {
        current_method_str = "SET_HURTBOX"; 
    } else if current_selection == 49 {
        current_method_str = "SET_INTERACT_TYPE"; 
    } else if current_selection == 50 {
        current_method_str = "SET_OBJ_PHYSICS"; 
    } else if current_selection == 51 {
        current_method_str = "SET_INTERACT_SUBTYPE"; 
    } else if current_selection == 52 {
        current_method_str = "SCALE"; 
    } else if current_selection == 53 {
        current_method_str = "PARENT_BIT_CLEAR"; 
    } else if current_selection == 54 {
        current_method_str = "ANIMATE_TEXTURE"; 
    } else if current_selection == 55 {
        current_method_str = "DISABLE_RENDERING"; 
    } else if current_selection == 56 {
        current_method_str = "SET_INT_UNUSED"; 
    } else if current_selection == 57 {
        current_method_str = "SPAWN_WATER_DROPLET"; 
    } else if current_selection == 58 {
        return Paragraph::new(vec![
            Line::from("0x1 Mario can hang from it                              0x200 Nothing (can be punched)".bold()),
            Line::from("0x2 Mario can pick it up                                0x400 Blows Mario away".bold()),
            Line::from("0x4 Door                                                0x800 Warp door".bold()),
            Line::from("0x8 Damages Mario (normal)                              0x1000 Star".bold()),
            Line::from("0x10 Coin                                               0x2000 Warp hole".bold()),
            Line::from("0x20 Cap                                                0x4000 Cannon".bold()),
            Line::from("0x40 Pole                                               0x8000 Damages Mario (can be punched, bounced on)".bold()),
            Line::from("0x80 Damages Mario (can be punched, bounced on)         0x10000 Replenishes health".bold()),
            Line::from("0x100 Damages Mario (can be punched)                    0x20000 Bully".bold()),
        ]);
    } else if current_selection == 59 {
        return Paragraph::new(vec![
            Line::from("0x40000 Flame                                           0x8000000 Warp (Mario shrinks in)".bold()),
            Line::from("0x80000 Koopa shell                                     0x10000000 Damages Mario".bold()),
            Line::from("0x100000 Damages Mario (can be punched, bounced on)     0x20000000 Electrocutes Mario".bold()),
            Line::from("0x200000 Damages Mario                                  0x40000000 Normal".bold()),
            Line::from("0x400000 Damages Mario (can be punched and bounced on)".bold()),
            Line::from("0x800000 Message".bold()),
            Line::from("0x1000000 Makes Mario spin".bold()),
            Line::from("0x2000000 Makes Mario fall?".bold()),
            Line::from("0x4000000 Damages Mario".bold()),
        ]);
    }

    else {
        return Paragraph::new(vec![Line::from("".light_red())]);
    }

    if current_method_str.contains("Error") {
        return Paragraph::new(vec![Line::from("Current line contains an error".light_red())]);
    } else if current_method_str == "" {
        return Paragraph::new(vec![Line::from("No current command".light_red())]);
    } else if current_method_str == "BEGIN" {
        return Paragraph::new(vec![
            Line::from("00 BEGIN".light_cyan()),
            Line::from("Marks start of behavior."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BEGIN(ObjectList)".bold()),
            Line::from("OBJ_LIST_PLAYER (0), OBJ_LIST_UNUSED_1 (1), OBJ_LIST_DESTRUCTIVE (2), OBJ_LIST_UNUSED_3 (3),".bold()),
            Line::from("OBJ_LIST_GENACTOR (4), OBJ_LIST_PUSHABLE (5), OBJ_LIST_LEVEL (6), OBJ_LIST_UNUSED_7 (7),".bold()),
            Line::from("OBJ_LIST_DEFAULT (8), OBJ_LIST_SURFACE (9), OBJ_LIST_POLELIKE (10), OBJ_LIST_SPAWNER (11),".bold()),
            Line::from("OBJ_LIST_UNIMPORTANT (12)".bold()),
        ]);
    } else if current_method_str == "DELAY" {
        return Paragraph::new(vec![
            Line::from("01 DELAY".light_cyan()),
            Line::from("Delays the behavior script for a certain number of frames."),
            Line::from("Syntax:".light_magenta()),
            Line::from("DELAY(AAAA)".bold()),
            Line::from("A - Frames".bold()),
        ]);
    } else if current_method_str == "CALL" {
        return Paragraph::new(vec![
            Line::from("02 CALL".light_cyan()),
            Line::from("Jumps to a new behavior command and stores the return address in the object's stack."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CALL(AAAAAAAA)".bold()),
            Line::from("A - Segmented address of behavior to jump to".bold()),
        ]);
    } else if current_method_str == "RETURN" {
        return Paragraph::new(vec![
            Line::from("03 RETURN".light_cyan()),
            Line::from("Jumps back to the behavior command stored in the object's stack."),
            Line::from("Syntax:".light_magenta()),
            Line::from("RETURN()".bold()),
        ]);
    } else if current_method_str == "GOTO" {
        return Paragraph::new(vec![
            Line::from("04 GOTO".light_cyan()),
            Line::from("Jumps to a new behavior script without saving anything."),
            Line::from("Syntax:".light_magenta()),
            Line::from("GOTO(AAAAAAAA)".bold()),
            Line::from("A - Segmented address of behavior to jump to".bold()),
        ]);
    } else if current_method_str == "BEGIN_REPEAT" {
        return Paragraph::new(vec![
            Line::from("05 BEGIN_REPEAT".light_cyan()),
            Line::from("Marks the start of a loop that will repeat a certain number of times."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BEGIN_REPEAT(AAAA)".bold()),
            Line::from("A - Number of times to loop".bold()),
        ]);
    } else if current_method_str == "END_REPEAT" {
        return Paragraph::new(vec![
            Line::from("06 END_REPEAT".light_cyan()),
            Line::from("Marks the end of a repeating loop."),
            Line::from("Syntax:".light_magenta()),
            Line::from("END_REPEAT()".bold()),
        ]);
    } else if current_method_str == "END_REPEAT_CONTINUE" {
        return Paragraph::new(vec![
            Line::from("07 END_REPEAT_CONTINUE".light_cyan()),
            Line::from("Marks the end of a repeating loop and continues executing"),
            Line::from("commands following the loop on the same frame."),
            Line::from("Syntax:".light_magenta()),
            Line::from("END_REPEAT_CONTINUE()".bold()),
        ]);
    } else if current_method_str == "BEGIN_LOOP" {
        return Paragraph::new(vec![
            Line::from("08 BEGIN_LOOP".light_cyan()),
            Line::from("Marks the beginning of an infinite loop."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BEGIN_LOOP()".bold()),
        ]);
    } else if current_method_str == "END_LOOP" {
        return Paragraph::new(vec![
            Line::from("09 END_LOOP".light_cyan()),
            Line::from("Marks the end of an infinite loop."),
            Line::from("Syntax:".light_magenta()),
            Line::from("END_LOOP()".bold()),
        ]);
    } else if current_method_str == "BREAK" {
        return Paragraph::new(vec![
            Line::from("0A BREAK".light_cyan()),
            Line::from("Exits the behavior script."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BREAK()".bold()),
        ]);
    } else if current_method_str == "BREAK_UNUSED" {
        return Paragraph::new(vec![
            Line::from("0B BREAK_UNUSED".light_cyan()),
            Line::from("Exits the behavior script (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BREAK_UNUSED()".bold()),
        ]);
    } else if current_method_str == "CALL_NATIVE" {
        return Paragraph::new(vec![
            Line::from("0C CALL_NATIVE".light_cyan()),
            Line::from("Calls an ASM function in RAM."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CALL_NATIVE(AAAAAAAA)".bold()),
            Line::from("A - RAM address of ASM function to call".bold()),
        ]);
    } else if current_method_str == "ADD_FLOAT" {
        return Paragraph::new(vec![
            Line::from("0D ADD_FLOAT".light_cyan()),
            Line::from("Used to offset the value of an address by a float."),
            Line::from("Syntax:".light_magenta()),
            Line::from("ADD_FLOAT(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Float (s16)".bold()),
        ]);
    } else if current_method_str == "SET_FLOAT" {
        return Paragraph::new(vec![
            Line::from("0E SET_FLOAT".light_cyan()),
            Line::from("Used to set the value of an address to a float."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_FLOAT(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Float (s16)".bold()),
        ]);
    } else if current_method_str == "ADD_INT" {
        return Paragraph::new(vec![
            Line::from("0F ADD_INT".light_cyan()),
            Line::from("Used to offset the value of an address by an integer."),
            Line::from("Syntax:".light_magenta()),
            Line::from("ADD_INT(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Integer (u16)".bold()),
        ]);
    } else if current_method_str == "SET_INT" {
        return Paragraph::new(vec![
            Line::from("10 SET_INT".light_cyan()),
            Line::from("Used to set the value of an address to an integer."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_INT(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Integer (u16)".bold()),
        ]);
    } else if current_method_str == "OR_INT" {
        return Paragraph::new(vec![
            Line::from("11 OR_INT".light_cyan()),
            Line::from("Sets bits designated by mask B at object offset A*4+0x88."),
            Line::from("Syntax:".light_magenta()),
            Line::from("OR_INT(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Integer (u16)".bold()),
        ]);
    } else if current_method_str == "BIT_CLEAR" {
        return Paragraph::new(vec![
            Line::from("12 BIT_CLEAR".light_cyan()),
            Line::from("Clears bits designated by mask B at object offset A*4+0x88."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BIT_CLEAR(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Integer (u16)".bold()),
        ]);
    } else if current_method_str == "SET_INT_RAND_RSHIFT" {
        return Paragraph::new(vec![
            Line::from("13 SET_INT_RAND_RSHIFT".light_cyan()),
            Line::from("Gets a random short, right shifts it by C and adds B to it,"),
            Line::from("then sets A*4+0x88 to that value."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_INT_RAND_RSHIFT(AA, BBBB, CCCC)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Integer to add (u16)".bold()),
            Line::from("C - Right shift (u16)".bold()),
        ]);
    } else if current_method_str == "SET_RANDOM_FLOAT" {
        return Paragraph::new(vec![
            Line::from("14 SET_RANDOM_FLOAT".light_cyan()),
            Line::from("Sets A*4+0x88 to a random float in the given range?"),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_RANDOM_FLOAT(AA, BBBB, CCCC)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Float (s16)".bold()),
            Line::from("C - Float (s16)".bold()),
        ]);
    } else if current_method_str == "SET_RANDOM_INT" {
        return Paragraph::new(vec![
            Line::from("15 SET_RANDOM_INT".light_cyan()),
            Line::from("Sets A*4+0x88 to a random integer in the given range?"),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_RANDOM_INT(AA, BBBB, CCCC)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Minimum (u16?)".bold()),
            Line::from("C - Range (u16?)".bold()),
        ]);
    } else if current_method_str == "ADD_RANDOM_FLOAT" {
        return Paragraph::new(vec![
            Line::from("16 ADD_RANDOM_FLOAT".light_cyan()),
            Line::from("Adds a random float to A*4+0x88 in the given range?"),
            Line::from("Syntax:".light_magenta()),
            Line::from("ADD_RANDOM_FLOAT(AA, BBBB, CCCC)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Float (s16)".bold()),
            Line::from("C - Float (s16)".bold()),
        ]);
    } else if current_method_str == "ADD_INT_RAND_RSHIFT" {
        return Paragraph::new(vec![
            Line::from("17 ADD_INT_RAND_RSHIFT".light_cyan()),
            Line::from("Gets a random short, right shifts it the specified amount and adds min to it,"),
            Line::from("then adds the value to A*4+0x88."),
            Line::from("Syntax:".light_magenta()),
            Line::from("ADD_INT_RAND_RSHIFT(AA, BBBB, CCCC)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Minimum (u16)".bold()),
            Line::from("C - Right Shift (u16)".bold()),
        ]);
    } else if current_method_str == "CMD_NOP_1" {
        return Paragraph::new(vec![
            Line::from("18 CMD_NOP_1".light_cyan()),
            Line::from("No operation (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CMD_NOP_1()".bold()),
        ]);
    } else if current_method_str == "CMD_NOP_2" {
        return Paragraph::new(vec![
            Line::from("19 CMD_NOP_2".light_cyan()),
            Line::from("No operation (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CMD_NOP_2()".bold()),
        ]);
    } else if current_method_str == "CMD_NOP_3" {
        return Paragraph::new(vec![
            Line::from("1A CMD_NOP_3".light_cyan()),
            Line::from("No operation (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CMD_NOP_3()".bold()),
        ]);
    } else if current_method_str == "SET_MODEL" {
        return Paragraph::new(vec![
            Line::from("1B SET_MODEL".light_cyan()),
            Line::from("Sets the current model ID of the object."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_MODEL(IIII)".bold()),
            Line::from("A - Model ID".bold()),
        ]);
    } else if current_method_str == "SPAWN_CHILD" {
        return Paragraph::new(vec![
            Line::from("1C SPAWN_CHILD".light_cyan()),
            Line::from("Spawns a child object with the specified model and behavior."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SPAWN_CHILD(IIIIIIII, AAAAAAAA)".bold()),
            Line::from("I - Model ID".bold()),
            Line::from("A - Segmented address of child object behavior".bold()),
        ]);
    } else if current_method_str == "DEACTIVATE" {
        return Paragraph::new(vec![
            Line::from("1D DEACTIVATE".light_cyan()),
            Line::from("Exits the behavior script and despawns the object."),
            Line::from("Syntax:".light_magenta()),
            Line::from("DEACTIVATE()".bold()),
        ]);
    } else if current_method_str == "DROP_TO_FLOOR" {
        return Paragraph::new(vec![
            Line::from("1E DROP_TO_FLOOR".light_cyan()),
            Line::from("Finds the floor triangle directly under the object and moves the object down to it."),
            Line::from("Syntax:".light_magenta()),
            Line::from("DROP_TO_FLOOR()".bold()),
        ]);
    } else if current_method_str == "SUM_FLOAT" {
        return Paragraph::new(vec![
            Line::from("1F SUM_FLOAT".light_cyan()),
            Line::from("Sets the destination float field to the sum of the values of the given float fields."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SUM_FLOAT(AA, BB, CC)".bold()),
            Line::from("A - Destination Address = A*4+0x88".bold()),
            Line::from("B - Address 1 = B*4+0x88".bold()),
            Line::from("C - Address 2 = C*4+0x88".bold()),
        ]);
    } else if current_method_str == "SUM_INT" {
        return Paragraph::new(vec![
            Line::from("20 SUM_INT".light_cyan()),
            Line::from("Sets the destination integer field to the sum of the values of the given integer fields (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SUM_INT(AA, BB, CC)".bold()),
            Line::from("A - Destination Address = A*4+0x88".bold()),
            Line::from("B - Address 1 = B*4+0x88".bold()),
            Line::from("C - Address 2 = C*4+0x88".bold()),
        ]);
    } else if current_method_str == "BILLBOARD" {
        return Paragraph::new(vec![
            Line::from("21 BILLBOARD".light_cyan()),
            Line::from("Billboards the current object, making it always face the camera."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BILLBOARD()".bold()),
        ]);
    } else if current_method_str == "HIDE" {
        return Paragraph::new(vec![
            Line::from("22 HIDE".light_cyan()),
            Line::from("Hides the current object."),
            Line::from("Syntax:".light_magenta()),
            Line::from("HIDE()".bold()),
        ]);
    } else if current_method_str == "SET_HITBOX" {
        return Paragraph::new(vec![
            Line::from("23 SET_HITBOX".light_cyan()),
            Line::from("Sets the size of the object's cylindrical hitbox."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_HITBOX(RRRR, HHHH)".bold()),
            Line::from("R - Radius of collision cylinder".bold()),
            Line::from("H - Height of collision cylinder".bold()),
        ]);
    } else if current_method_str == "CMD_NOP_4" {
        return Paragraph::new(vec![
            Line::from("24 CMD_NOP_4".light_cyan()),
            Line::from("No operation (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("CMD_NOP_4(AA, BBBB)".bold()),
        ]);
    } else if current_method_str == "DELAY_VAR" {
        return Paragraph::new(vec![
            Line::from("25 DELAY_VAR".light_cyan()),
            Line::from("Delays the behavior script for the number of frames given by the value of the specified field."),
            Line::from("Syntax:".light_magenta()),
            Line::from("DELAY_VAR(AA)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
        ]);
    } else if current_method_str == "BEGIN_REPEAT_UNUSED" {
        return Paragraph::new(vec![
            Line::from("26 BEGIN_REPEAT_UNUSED".light_cyan()),
            Line::from("Marks the start of a loop that will repeat a certain number of times."),
            Line::from("Syntax:".light_magenta()),
            Line::from("BEGIN_REPEAT_UNUSED(AA)".bold()),
            Line::from("A - Loops (u8)".bold()),
        ]);
    } else if current_method_str == "LOAD_ANIMATIONS" {
        return Paragraph::new(vec![
            Line::from("27 LOAD_ANIMATIONS".light_cyan()),
            Line::from("Loads the animations for the object. <field> is always set to oAnimations."),
            Line::from("Syntax:".light_magenta()),
            Line::from("LOAD_ANIMATIONS(AA, BBBBBBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Word to store at A*4+0x88".bold()),
        ]);
    } else if current_method_str == "ANIMATE" {
        return Paragraph::new(vec![
            Line::from("28 ANIMATE".light_cyan()),
            Line::from("Marks the start of a loop that will repeat a certain number of times."),
            Line::from("Syntax:".light_magenta()),
            Line::from("ANIMATE(AA)".bold()),
            Line::from("A - Animation index (*4)".bold()),
        ]);
    } else if current_method_str == "SPAWN_CHILD_WITH_PARAM" {
        return Paragraph::new(vec![
            Line::from("29 SPAWN_CHILD_WITH_PARAM".light_cyan()),
            Line::from("Spawns a child object with the specified model and behavior, plus a behavior param."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SPAWN_CHILD_WITH_PARAM(AAAA, BBBBBBBB, CCCCCCCC)".bold()),
            Line::from("A - BParam for child object".bold()),
            Line::from("B - Model ID".bold()),
            Line::from("C - Segmented address of behavior".bold()),
        ]);
    } else if current_method_str == "LOAD_COLLISION_DATA" {
        return Paragraph::new(vec![
            Line::from("2A LOAD_COLLISION_DATA".light_cyan()),
            Line::from("Loads collision data for the object."),
            Line::from("Syntax:".light_magenta()),
            Line::from("LOAD_COLLISION_DATA(AAAAAAAA)".bold()),
            Line::from("A - Segmented address of collision pointer".bold()),
        ]);
    } else if current_method_str == "SET_HITBOX_WITH_OFFSET" {
        return Paragraph::new(vec![
            Line::from("2B SET_HITBOX_WITH_OFFSET".light_cyan()),
            Line::from("Sets the size of the object's cylindrical hitbox, and applies a downwards offset."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_HITBOX_WITH_OFFSET(RRRR, HHHH, AAAA)".bold()),
            Line::from("R - Radius of collision cylinder".bold()),
            Line::from("H - Height of collision cylinder".bold()),
            Line::from("A - Downwards offset".bold()),
        ]);
    } else if current_method_str == "SPAWN_OBJ" {
        return Paragraph::new(vec![
            Line::from("2C SPAWN_OBJ".light_cyan()),
            Line::from("Spawns a new object with the specified model and behavior."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SPAWN_OBJ(IIIIIIII, AAAAAAAA)".bold()),
            Line::from("I - Model ID".bold()),
            Line::from("A - Segmented address of behavior".bold()),
        ]);
    } else if current_method_str == "SET_HOME" {
        return Paragraph::new(vec![
            Line::from("2D SET_HOME".light_cyan()),
            Line::from("Sets the home position of the object to its current position."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_HOME()".bold()),
        ]);
    } else if current_method_str == "SET_HURTBOX" {
        return Paragraph::new(vec![
            Line::from("2E SET_HURTBOX".light_cyan()),
            Line::from("Sets the size of the object's cylindrical hurtbox."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_HURTBOX(RRRR, HHHH)".bold()),
            Line::from("R - Radius of collision cylinder".bold()),
            Line::from("H - Height of collision cylinder".bold()),
        ]);
    } else if current_method_str == "SET_INTERACT_TYPE" {
        return Paragraph::new(vec![
            Line::from("2F SET_INTERACT_TYPE".light_cyan()),
            Line::from("Sets the object's interaction type."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_INTERACT_TYPE(AAAAAAAA)".bold()),
            Line::from("A - Interaction type (see Help List -> Interaction Types)".bold()),
        ]);
    } else if current_method_str == "SET_OBJ_PHYSICS" {
        return Paragraph::new(vec![
            Line::from("30 SET_OBJ_PHYSICS".light_cyan()),
            Line::from("Sets the object's interaction type."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_OBJ_PHYSICS(AAAA, BBBB, CCCC, DDDD, EEEE, FFFF, GGGG, HHHH)".bold()),
            Line::from("A - Wall hitbox radius      E - Friction".bold()),
            Line::from("B - Gravity                 F - Buoyancy".bold()),
            Line::from("C - Bounce                  G - Ignored".bold()),
            Line::from("D - Drag strength           H - Ignored".bold()),
        ]);
    } else if current_method_str == "SET_INTERACT_SUBTYPE" {
        return Paragraph::new(vec![
            Line::from("31 SET_INTERACT_SUBTYPE".light_cyan()),
            Line::from("Sets the object's interaction subtype (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_INTERACT_SUBTYPE(AAAAAAAA)".bold()),
            Line::from("A - Interaction subtype".bold()),
        ]);
    } else if current_method_str == "SCALE" {
        return Paragraph::new(vec![
            Line::from("32 SCALE".light_cyan()),
            Line::from("Sets the object's scale to the specified percentage."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SCALE(AAAA)".bold()),
            Line::from("A - Scale value (percent)".bold()),
        ]);
    } else if current_method_str == "PARENT_BIT_CLEAR" {
        return Paragraph::new(vec![
            Line::from("33 PARENT_BIT_CLEAR".light_cyan()),
            Line::from("Performs a bit clear on the object's parent's field with the specified value."),
            Line::from("Syntax:".light_magenta()),
            Line::from("PARENT_BIT_CLEAR(AA, BBBBBBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Bit values to clear".bold()),
        ]);
    } else if current_method_str == "ANIMATE_TEXTURE" {
        return Paragraph::new(vec![
            Line::from("34 ANIMATE_TEXTURE".light_cyan()),
            Line::from("Animates an object using texture animation. <field> is always set to oAnimState."),
            Line::from("Syntax:".light_magenta()),
            Line::from("ANIMATE_TEXTURE(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Rate; divide value at 0x8032D5D4 with B".bold()),
        ]);
    } else if current_method_str == "DISABLE_RENDERING" {
        return Paragraph::new(vec![
            Line::from("35 DISABLE_RENDERING".light_cyan()),
            Line::from("Disables rendering for the object."),
            Line::from("Syntax:".light_magenta()),
            Line::from("DISABLE_RENDERING()".bold()),
        ]);
    } else if current_method_str == "SET_INT_UNUSED" {
        return Paragraph::new(vec![
            Line::from("36 SET_INT_UNUSED".light_cyan()),
            Line::from("Sets the specified field to an integer (unused)."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SET_INT_UNUSED(AA, BBBB)".bold()),
            Line::from("A - Address = A*4+0x88".bold()),
            Line::from("B - Value (u16)".bold()),
        ]);
    } else if current_method_str == "SPAWN_WATER_DROPLET" {
        return Paragraph::new(vec![
            Line::from("37 SPAWN_WATER_DROPLET".light_cyan()),
            Line::from("Spawns a water droplet with the given parameters."),
            Line::from("Syntax:".light_magenta()),
            Line::from("SPAWN_WATER_DROPLET(AA, BBBB)".bold()),
            Line::from("A - Spawn function address/droplet params?".bold()),
        ]);
    }
    return Paragraph::new(vec![Line::from("Unknown command".light_red())]);
}

fn get_current_method(mut app_state: &mut AppState) {
    let mut line_index: usize = 0;
    for line in app_state.decomp_text_area.lines() {
        if line_index == app_state.binary_text_area.cursor().0 {
            if *line == "".to_string() {
                line_index += 1;
                continue;
            }

            if !line.contains("(") || !line.contains(")") {
                line_index += 1;
                continue; 
            }

            let parenthese_split: Vec<String> = split_to_vec!(line.split("("));

            let method = parenthese_split[0].as_str();

            app_state.current_method = method.to_string();
        }
        line_index += 1;
    }
}

fn update_atimes4(mut app_state: &mut AppState) {
    let mut first_line = String::new();
    for line in app_state.atimes4_text_area.lines() {
        first_line = line.trim_start_matches("0x").trim_start_matches("0X").to_string();
        break;
    }
    if first_line == "".to_string() {
        app_state.single_byte_address = "".to_string();
        return;
    }
    let first_line_int = match i64::from_str_radix(&first_line, 16) {
        Ok(val) => val,
        Err(_) => { 
            app_state.single_byte_address = "Error: couldn't convert to hex".to_string();
            return;
        }
    };
    let result = (first_line_int - 136) / 4;
    let result_hex = format!("{:x}", &result);
    let mut result_string = "- 0x88 / 4 = 0x".to_string();
    let result_hex_length = result_hex.chars().count();
    if result_hex_length == 1 {
        result_string.push_str("0");
        result_string.push_str(&result_hex);
        app_state.single_byte_address = result_string;
    } else if result_hex_length == 2 {
        result_string.push_str(&result_hex);
        app_state.single_byte_address = result_string;
    } else {
        app_state.single_byte_address = "Error: couldn't convert the offset".to_string();
    }
}

fn clear_flag(mut app_state: &mut AppState) {
    let mut object_flags = String::new();
    for line in app_state.object_flags_text_area.lines() {
        object_flags = line.to_string();
        break;
    }
    let cursor_x = app_state.object_flags_text_area.cursor().1;
    object_flags.replace_range(cursor_x..cursor_x+1, "0");
    app_state.object_flags_text_area = TextArea::from(vec![&object_flags]);
    app_state.object_flags_text_area.move_cursor(CursorMove::Jump(0, cursor_x as u16));
}

fn set_flag(mut app_state: &mut AppState) {
    let mut object_flags = String::new();
    for line in app_state.object_flags_text_area.lines() {
        object_flags = line.to_string();
        break;
    }
    let cursor_x = app_state.object_flags_text_area.cursor().1;
    object_flags.replace_range(cursor_x..cursor_x+1, "1");
    app_state.object_flags_text_area = TextArea::from(vec![&object_flags]);
    app_state.object_flags_text_area.move_cursor(CursorMove::Jump(0, cursor_x as u16));
}

fn calculate_flags(app_state: &AppState, copy_format: bool) -> String {
    let mut object_flags = String::new();
    for line in app_state.object_flags_text_area.lines() {
        object_flags = line.to_string();
        break;
    }
    let result = match i64::from_str_radix(&object_flags, 2) {
        Ok(val) => val,
        Err(_) => {
            return "Error: couldn't convert binary".to_string();
        }
    };
    let result_hex = format!("{:x}", &result).to_uppercase();
    let mut result_string;
    if !copy_format { result_string = "Command: OR_INT(0x1, 0x".to_string(); }
    else            { result_string = "OR_INT(0x1, 0x".to_string(); }
    result_string.push_str(&result_hex);
    result_string.push_str(")");
    return result_string;
}

fn get_flag_name(app_state: &AppState) -> String {
    let cursor_x = app_state.object_flags_text_area.cursor().1;
    match cursor_x {
        0 => { "OBJ_FLAG_8000".to_string() },
        1 => { "OBJ_FLAG_PERSISTENT_RESPAWN".to_string() },
        2 => { "OBJ_FLAG_COMPUTE_ANGLE_TO_MARIO".to_string() },
        3 => { "OBJ_FLAG_1000".to_string() },
        4 => { "OBJ_FLAG_SET_THROW_MATRIX_FROM_TRANSFORM".to_string() },
        5 => { "OBJ_FLAG_HOLDABLE".to_string() },
        6 => { "OBJ_FLAG_TRANSFORM_RELATIVE_TO_PARENT".to_string() },
        7 => { "OBJ_FLAG_0100".to_string() },
        8 => { "OBJ_FLAG_ACTIVE_FROM_AFAR".to_string() },
        9 => { "OBJ_FLAG_COMPUTE_DIST_TO_MARIO".to_string() },
        10 => { "OBJ_FLAG_0020".to_string() },
        11 => { "OBJ_FLAG_SET_FACE_ANGLE_TO_MOVE_ANGLE".to_string() },
        12 => { "OBJ_FLAG_SET_FACE_YAW_TO_MOVE_YAW".to_string() },
        13 => { "OBJ_FLAG_MOVE_Y_WITH_TERMINAL_VEL".to_string() },
        14 => { "OBJ_FLAG_MOVE_XZ_USING_FVEL".to_string() },
        15 => { "OBJ_FLAG_UPDATE_GFX_POS_AND_ANGLE".to_string() },
        _ => { "Error: index out of bounds".to_string() }
    }
}
