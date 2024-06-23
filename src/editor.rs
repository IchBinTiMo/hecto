use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{env, io::Error, panic::{set_hook, take_hook}
};

mod command;
mod commandbar;
mod documentstatus;
mod line;
mod messagebar;
mod position;
mod size;
mod statusbar;
mod terminal;
mod uicomponent;
mod view;

use self::command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewline,
    System::{Dismiss, Quit, Resize, Save, Search},
};
use commandbar::CommandBar;
use documentstatus::DocumentStatus;
use line::Line;
use messagebar::MessageBar;
use position::Position;
use size::Size;
use statusbar::StatusBar;
use terminal::Terminal;
use uicomponent::UIComponent;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3;

#[derive(Eq, PartialEq, Default)]
enum PromptType {
    #[default]
    None,
    Search,
    Save,
}

impl PromptType {
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

#[derive(Default)]
pub struct Editor {
    should_exit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: CommandBar,
    prompt_type: PromptType,
    // command_bar: Option<CommandBar>,
    terminal_size: Size,
    title: String,
    quit_times: u8,
}

impl Editor {
    // SECTION: struct lifecycle

    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        let args: Vec<String> = env::args().collect();

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();

        editor.handle_resize_command(size);
        editor.update_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        // editor

        // editor.resize(size);

        // editor
        //     .message_bar
        //     .update_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");

        if let Some(file_name) = args.get(1) {
            if editor.view.load_file(file_name).is_err() {
                editor.update_message(&format!("ERR: Could not open file: {file_name}"));
                // editor
                //     .message_bar
                //     .update_message(&format!("ERR: Could not open file: {file_name}"));
            }
        }

        editor.refrest_status();

        Ok(editor)
    }

    // END SECTION

    // SECTION: event loop

    // fn resize(&mut self, size: Size) {
    //     self.terminal_size = size;
    //     self.view.resize(Size {
    //         height: size.height.saturating_sub(2),
    //         width: size.width,
    //     });

    //     self.message_bar.resize(Size {
    //         height: 1,
    //         width: size.width,
    //     });

    //     self.status_bar.resize(Size {
    //         height: 1,
    //         width: size.width,
    //     });

    //     if let Some(command_bar) = &mut self.command_bar {
    //         command_bar.resize(Size {
    //             height: 1,
    //             width: size.width,
    //         });
    //     }
    // }

    
    pub fn run(&mut self) {
        loop {
            self.refresh_screen();

            if self.should_exit {
                break;
            }

            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }

            self.refrest_status();
            // let status: DocumentStatus = self.view.get_status();
            // self.status_bar.update_status(status);
        }
    }

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        if self.in_prompt() {
            self.command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }

        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };

        let _ = Terminal::move_caret_to(new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    fn refrest_status(&mut self) {
        let status: DocumentStatus = self.view.get_status();
        let title: String = format!("{} - {NAME}", status.file_name);

        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
            }
        }
    }

    // ENDSECTION

    // SECTION: commands handling

    fn process_command(&mut self, command: Command) {
        if let System(Resize(size)) = command {
            self.handle_resize_command(size);
            return;
        }
        match self.prompt_type {
            PromptType::Search => self.process_command_during_search(command),
            PromptType::Save => self.process_command_during_save(command),
            PromptType::None => self.process_command_no_prompt(command),
        }
        // match command {
        //     System(Quit) => {
        //         if self.command_bar.is_none() {
        //             self.handle_quit_command();
        //         }
        //     }
        //     System(Resize(size)) => self.resize(size),
        //     _ => self.reset_quit_times(),
        // }

        // match command {
        //     System(Quit | Resize(_)) => {}
        //     System(Save) => {
        //         if self.command_bar.is_none() {
        //             self.handle_save_command();
        //         }
        //     }
        //     System(Search) => {
        //         if self.command_bar.is_none() {
        //             self.handle_search();
        //         }
        //     }
        //     System(Dismiss) => {
        //         if self.command_bar.is_some() {
        //             self.dismiss_prompt();
        //             // self.message_bar.update_message("Save aborted.");
        //         }
        //     }
        //     Edit(edit_command) => {
        //         if let Some(command_bar) = &mut self.command_bar {
        //             if matches!(edit_command, InsertNewline) {
        //                 if command_bar.prompt() == "Save as: " {
        //                     let file_name: String = command_bar.value();
        //                     self.dismiss_prompt();
        //                     self.save_file(Some(&file_name));
        //                 }
        //             } else {
        //                 // if command_bar.prompt() == "Search: " {
        //                 //     let to_search: String = command_bar.value();
        //                 //     // self.dismiss_prompt();
        //                 //     self.search(to_search);
        //                 // }
        //                 command_bar.handle_edit_command(edit_command);
        //             }
        //         } else {
        //             self.view.handle_edit_command(edit_command);
        //         }
        //     }
        //     Move(move_command) => {
        //         if let Some(command_bar) = &mut self.command_bar {
        //             // self.view.handle_move_command(move_command, true);
        //             command_bar.handle_move_command(move_command);
        //         } else {
        //             self.view.handle_move_command(move_command);
        //         }
        //         // if self.command_bar.is_none() {
        //         //     self.view.handle_move_command(move_command);
        //         // } else {
                    
        //         // }
        //     }
        // }
    }

    fn process_command_no_prompt(&mut self, command: Command) {
        if matches!(command, System(Quit)) {
            self.handle_quit_command();
            return;
        }

        self.reset_quit_times();

        match command {
            System(Quit | Resize(_) | Dismiss) => {},
            System(Save) => self.handle_save_command(),
            System(Search) => self.set_prompt(PromptType::Search),
            Edit(edit_command) => self.view.handle_edit_command(edit_command),
            Move(move_command) => self.view.handle_move_command(move_command),
        }
    }

    // fn dismiss_prompt(&mut self) {
    //     if let Some(command_bar) = self.command_bar.take() {
    //         if command_bar.prompt() == "Save as: " {
    //             self.message_bar.update_message("Save aborted.");
    //         } else if command_bar.prompt() == "Search: " {
    //             self.message_bar.update_message("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
    //         }
    //     }
    //     // self.command_bar = None;
    //     self.message_bar.set_needs_redraw(true);
    // }

    // fn show_prompt(&mut self, prompt: &str) {
    //     let mut command_bar = CommandBar::default();

    //     command_bar.set_prompt(prompt);
    //     command_bar.set_caret_postion(prompt.len());
    //     command_bar.resize(Size {
    //         height: 1,
    //         width: self.terminal_size.width,
    //     });
    //     command_bar.set_needs_redraw(true);
    //     // self.command_bar = Some(command_bar);
    // }

    // SECTION: save command and prompt handling

    fn handle_save_command(&mut self) {
        if self.view.is_file_loaded() {
            self.save_file(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    fn process_command_during_save(&mut self, command: Command) {
        match command {
            System(Quit | Resize(_) | Search | Save) => {},
            System(Dismiss) => {
                self.set_prompt(PromptType::None);
                self.update_message("Save aborted.");
            },
            Edit(InsertNewline) => {
                let file_name = self.command_bar.value();
                self.save_file(Some(&file_name));
                self.set_prompt(PromptType::None);
            },
            Edit(edit_command) => self.command_bar.handle_edit_command(edit_command),
            Move(move_command) => self.command_bar.handle_move_command(move_command),
        }
    }

    fn save_file(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save_file()
        };

        if result.is_ok() {
            self.message_bar.update_message("File saved successfully");
        } else {
            self.message_bar.update_message("Could not save file");
        }
    }

    // END SECTION

    // SECTION: search command and prompt handling

    fn process_command_during_search(&mut self, command: Command) {
        match command {
            System(Quit | Resize(_) | Search | Save) => {},
            System(Dismiss) => {self.set_prompt(PromptType::None); self.view.dismiss_search();},
            Edit(InsertNewline) => {
                self.set_prompt(PromptType::None);
                self.view.exit_search();
            }
            Edit(edit_command) => {
                self.command_bar.handle_edit_command(edit_command);
                let query = self.command_bar.value();
                self.view.search(&query);
            },
            Move(move_command) => self.command_bar.handle_move_command(move_command),
        }
    }

    // END SECTION

    // SECTION: quit command handling

    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit_command(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_exit = true;
        } else if self.view.get_status().is_modified {
            self.update_message(&format!(
                "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.",
                QUIT_TIMES - self.quit_times - 1
            ));

            self.quit_times += 1;
        }
    }

    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.message_bar.update_message("");
        }
    }

    // END SECTION

    // SECTION: message and command bar

    fn update_message(&mut self, new_message: &str) {
        self.message_bar.update_message(new_message);
    }

    // END SECTION

    // SECTION: resize command handling
    fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });

        let bar_size = Size {
            height: 1,
            width: size.width,
        };

        self.message_bar.resize(bar_size);
        self.status_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    }

    // END SECTION

    // SECTION: prompt handling
    fn in_prompt(&self) -> bool {
        !self.prompt_type.is_none()
    }

    fn set_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::None => self.message_bar.set_needs_redraw(true),
            PromptType::Save => self.command_bar.set_prompt("Save as: "),
            PromptType::Search => {
                self.command_bar.set_prompt("Search (Esc to cancel): ");
                self.view.enter_search();
            },
        }

        // self.command_bar.set_caret_postion()
        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
    }

    // END SECTION
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();

        if self.should_exit {
            let _ = Terminal::print("Farewell!!!\r\n");
        }
    }
}
