use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};

mod flieinfo;
mod documentstatus;
mod statusbar;
mod editorcommand;
mod terminal;
mod view;

// use flieinfo::FileInfo;
use documentstatus::DocumentStatus;
use statusbar::StatusBar;
use editorcommand::EditorCommand;
use terminal::Terminal;
use view::View;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_exit: bool,
    view: View,
    status_bar: StatusBar,
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

        
        let mut editor: Editor = Self {
            should_exit: false,
            view: View::new(2),
            status_bar: StatusBar::new(1),
            title: String::new()
        };
        
        if let Some(file_name) = args.get(1) {
            editor.view.load_file(file_name);
        }

        editor.refrest_status();

        Ok(editor)
    }

    pub fn refrest_status(&mut self) {
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
                } else {
                    self.view.handle_command(command);
                    if let EditorCommand::Resize(size) = command {
                        self.status_bar.resize(size);
                    }
                }
            }
        }
        
    }

    

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        self.view.render();
        self.status_bar.render();

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
