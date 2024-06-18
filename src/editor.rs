use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod uicomponent;
mod messagebar;
mod flieinfo;
mod documentstatus;
mod statusbar;
mod editorcommand;
mod terminal;
mod view;

// use flieinfo::FileInfo;
use self::{messagebar::MessageBar, terminal::Size};
use uicomponent::UIComponent;
use documentstatus::DocumentStatus;
use statusbar::StatusBar;
use editorcommand::EditorCommand;
use terminal::Terminal;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct Editor {
    should_exit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    terminal_size: Size,
    title: String,
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
        // let mut editor: Editor = Self {
        //     should_exit: false,
        //     view: View::new(2),
        //     status_bar: StatusBar::new(1),
        //     title: String::new()
        // };
        
        if let Some(file_name) = args.get(1) {
            editor.view.load_file(file_name);
        }

        editor.message_bar.update_message("HELP: Ctrl-S = save | Ctrl-Q = quit".to_string());

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

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = EditorCommand::try_from(event) {
                if matches!(command, EditorCommand::Quit) {
                    self.should_exit = true;
                } else if let EditorCommand::Resize(size) = command {
                    self.resize(size);
                } else {
                    self.view.handle_command(command);
                }
            }
        }
        
    }

    

    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        self.message_bar.render(self.terminal_size.height.saturating_sub(1));

        if self.terminal_size.height > 1 {
            self.status_bar.render(self.terminal_size.height.saturating_sub(2));
        }

        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        // self.view.render();
        // self.status_bar.render();

        let _ = Terminal::move_caret_to(self.view.caret_position());

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
