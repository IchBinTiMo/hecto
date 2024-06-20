// use command::Command;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod commandbar;
mod line;
mod position;
mod size;
mod uicomponent;
mod messagebar;
// mod flieinfo;
mod documentstatus;
mod statusbar;
mod command;
// mod editorcommand;
mod terminal;
mod view;

// use flieinfo::FileInfo;
use uicomponent::UIComponent;
// use documentstatus::DocumentStatus;
use statusbar::StatusBar;
// use self::{
//     command::{
//         Command::{self, Edit, Move, System},
//         System::{Quit, Resize, Save},
//     },
//     // Command::{self, Edit, Move, System},
//     // System::{Quit, Resize, Save},
//     messagebar::MessageBar, terminal::Size
// };
// use editorcommand::EditorCommand;
use self::command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewline,
    System::{Dismiss, Quit, Resize, Save},
};
use commandbar::CommandBar;
use documentstatus::DocumentStatus;
use line::Line;
use messagebar::MessageBar;
use position::Position;
use size::Size;
use terminal::Terminal;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 3;

#[derive(Default)]
pub struct Editor {
    should_exit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    command_bar: Option<CommandBar>,
    terminal_size: Size,
    title: String,
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));

        Terminal::initialize()?;

        // let mut view: View = View::new(2);

        let args: Vec<String> = env::args().collect();

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();

        editor.resize(size);

        editor.message_bar.update_message("HELP: Ctrl-S = save | Ctrl-Q = quit");
        // let mut editor: Editor = Self {
        //     should_exit: false,
        //     view: View::new(2),
        //     status_bar: StatusBar::new(1),
        //     title: String::new()
        // };
        
        if let Some(file_name) = args.get(1) {
            if editor.view.load_file(file_name).is_err() {
                editor.message_bar.update_message(&format!("ERR: Could not open file: {file_name}"));
            };
        }

        // editor.message_bar.update_message("HELP: Ctrl-S = save | Ctrl-Q = quit".to_string());

        editor.refrest_status();

        Ok(editor)
    }

    fn resize(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });

        self.message_bar.resize(Size {
            height: 1,
            width: size.width,
        });

        self.status_bar.resize(Size {
            height: 1,
            width: size.width,
        });

        if let Some(command_bar) = &mut self.command_bar {
            command_bar.resize(Size {
                height: 1,
                width: size.width,
            });
        }
    }

    fn refrest_status(&mut self) {
        let status: DocumentStatus = self.view.get_status();
        let title: String = format!("{} - {NAME}", status.file_name);

        self.status_bar.update_status(status);

        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }
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

            let status: DocumentStatus = self.view.get_status();
            self.status_bar.update_status(status);
        }
    }

    // #[allow(clippy::needless_pass_by_value)]
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
            // if let Ok(command) = EditorCommand::try_from(event) {
            //     if matches!(command, EditorCommand::Quit) {
            //         self.should_exit = true;
            //     } else if let EditorCommand::Resize(size) = command {
            //         self.resize(size);
            //     } else {
            //         self.view.handle_command(command);
            //     }
            // }
        }
        
    }

    fn process_command(&mut self, command: Command) {
        match command {
            System(Quit) => {
                if self.command_bar.is_none() {
                    self.handle_quit();
                }
            },
            System(Resize(size)) => self.resize(size),
            _ => self.reset_quit_times(),
        }

        match command {
            System(Quit | Resize(_)) => {},
            System(Save) => {
                if self.command_bar.is_none() {
                    self.handle_save();
                }
            },
            System(Dismiss) => {
                if self.command_bar.is_some() {
                    self.dismiss_prompt();
                    self.message_bar.update_message("Save aborted.");
                }
            },
            Edit(edit_command) => {
                if let Some(command_bar) = &mut self.command_bar {
                    if matches!(edit_command, InsertNewline) {
                        let file_name = command_bar.value();
                        self.dismiss_prompt();
                        self.save_file(Some(&file_name));
                    } else {
                        command_bar.handle_edit_command(edit_command);
                    }
                } else {
                    self.view.handle_edit_command(edit_command);
                }
            },
            Move(move_command) => {
                if self.command_bar.is_none() {
                    self.view.handle_move_command(move_command);
                }
            },
        }
    }

    fn dismiss_prompt(&mut self) {
        self.command_bar = None;
        self.message_bar.set_needs_redraw(true);
    }

    fn show_prompt(&mut self) {
        let mut command_bar = CommandBar::default();

        command_bar.set_prompt("Save as:");
        command_bar.resize(Size {
            height: 1,
            width: self.terminal_size.width,
        });
        command_bar.set_needs_redraw(true);
        self.command_bar = Some(command_bar);
    }

    fn handle_save(&mut self) {
        // if self.view.save_file().is_ok() {
        //     self.message_bar.update_message("File saved successfully");
        // } else {
        //     self.message_bar.update_message("Could not save file");
        // }
        if self.view.is_file_loaded() {
            self.save_file(None);
        } else {
            self.show_prompt();
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

    #[allow(clippy::arithmetic_side_effects)]
    fn handle_quit(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_exit = true;
        } else if self.view.get_status().is_modified {
            self.message_bar.update_message(&format!(
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

    

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        if let Some(command_bar) = &mut self.command_bar {
            command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }
        // self.message_bar.render(self.terminal_size.height.saturating_sub(1));

        if self.terminal_size.height > 1 {
            self.status_bar.render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }

        let new_caret_pos = if let Some(command_bar) = &self.command_bar {
            Position {
                row: bottom_bar_row,
                col: command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };
        // self.view.render();
        // self.status_bar.render();

        let _ = Terminal::move_caret_to(new_caret_pos);

        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();

        if self.should_exit {
            let _ = Terminal::print("Farewell!!!\r\n");
        }
    }
}
