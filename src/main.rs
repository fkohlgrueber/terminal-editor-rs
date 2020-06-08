use std::io::{Write, stdout};
use clap::{App, Arg}; 
use crossterm::{
    cursor,
    execute, queue,
    event::{Event, KeyCode, KeyModifiers},
    style::{Print, SetForegroundColor, SetBackgroundColor, Color},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

struct Model {
    exit: bool,
    orig: Option<String>,
    dimensions: (usize, usize),
    file_path: String,
    content: Vec<Vec<char>>,
    cursor: (usize, usize),
}

impl Model {
    fn update(&mut self, evt: Event) {
        match evt {
            Event::Key(key_event) => {
                match &(key_event.code, key_event.modifiers) {
                    // exit
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => self.exit = true,
                    
                    // save
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        let s = self.content_to_string();
                        if let Ok(_) = std::fs::write(&self.file_path, &s) {
                            self.orig = Some(s);
                        }
                    }

                    // char input
                    (KeyCode::Char(c), _) => {
                        let x = std::cmp::min(self.cursor.0, self.content[self.cursor.1].len());
                        self.content[self.cursor.1].insert(x, *c);
                        self.cursor.0 = x + 1;
                    }

                    // cursor motion
                    (KeyCode::Left, KeyModifiers::NONE) => {
                        let curr_line_len = self.content[self.cursor.1].len();
                        self.cursor.0 = std::cmp::min(self.cursor.0, curr_line_len);
                        if self.cursor.0 > 0 {
                            self.cursor.0 -= 1;
                        } else if self.cursor.0 == 0 && self.cursor.1 > 0 {
                            self.cursor.1 -= 1;
                            self.cursor.0 = self.content[self.cursor.1].len();
                        } 
                    }
                    (KeyCode::Right, KeyModifiers::NONE) => {
                        let curr_line_len = self.content[self.cursor.1].len();
                        self.cursor.0 = std::cmp::min(self.cursor.0, curr_line_len);
                        if self.cursor.0 < curr_line_len {
                            self.cursor.0 += 1;
                        } else if self.cursor.0 == curr_line_len && self.cursor.1 < self.content.len() - 1 {
                            self.cursor.0 = 0;
                            self.cursor.1 += 1;
                        }
                    }
                    (KeyCode::Up, KeyModifiers::NONE) => {
                        if self.cursor.1 > 0 {
                            self.cursor.1 -= 1;
                        } else {
                            self.cursor = (0, 0);
                        }
                    }
                    (KeyCode::Down, KeyModifiers::NONE) => {
                        let num_lines = self.content.len();
                        if self.cursor.1 < num_lines - 1 {
                            self.cursor.1 += 1;
                        } else {
                            self.cursor = (self.content.last().unwrap().len(), num_lines - 1);
                        }
                    }
                    (KeyCode::Home, KeyModifiers::NONE) => {
                        self.cursor.0 = 0;
                    }
                    (KeyCode::End, KeyModifiers::NONE) => {
                        self.cursor.0 = self.content[self.cursor.1].len();
                    }

                    // non-char keys
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        if self.cursor.0 > 0 {
                            self.content[self.cursor.1].remove(self.cursor.0 - 1);
                            self.cursor.0 -= 1;
                        } else if self.cursor.1 > 0 {
                            // join lines n-1 and n
                            let prev_line_width = self.content[self.cursor.1 - 1].len();
                            let s = self.content.remove(self.cursor.1);
                            self.content[self.cursor.1 - 1].extend(s.iter());
                            self.cursor = (prev_line_width, self.cursor.1 - 1)
                        }
                    }
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        let x = std::cmp::min(self.cursor.0, self.content[self.cursor.1].len());
                        let s = self.content[self.cursor.1][x..].to_owned();
                        self.content[self.cursor.1].truncate(x);
                        self.content.insert(self.cursor.1 + 1, s);
                        self.cursor.1 += 1;
                        self.cursor.0 = 0;
                    }

                    _ => { /* ignore */ }
                }
            }
            Event::Resize(x, y) => {
                self.dimensions = (x as usize, y as usize);
            }
            Event::Mouse(_) => {
                // don't handle mouse events
            }
        }
    }
    
    fn view(&self) -> crossterm::Result<()> {
        // header
        queue!(
            stdout(),

            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::White),
        )?;
        let is_dirty = match &self.orig {
            Some(s) => &self.content_to_string() != s,
            None => true
        };
        self.print_line(&format!("  {} {}", self.file_path, if is_dirty {'*'} else {' '}))?;
        
        // content
        queue!(
            stdout(),

            SetForegroundColor(Color::Reset),
            SetBackgroundColor(Color::Reset),
        )?;
        for i in 0..(self.dimensions.1 - 2) {
            let line_str = self.content.get(i)
                .map(|line| line.iter().take(self.dimensions.0).collect())
                .unwrap_or("".to_string());
            self.print_line(&line_str)?;
        }

        // footer
        queue!(
            stdout(),

            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::White),
        )?;
        self.print_line(&format!("  Ctrl-Q: Quit  Ctrl-S: Save"))?;
        
        // move cursor
        queue!(
            stdout(),
            
            cursor::MoveTo(
                std::cmp::min(
                    self.cursor.0, 
                    self.content[self.cursor.1].len()) as u16, 
                self.cursor.1 as u16 + 1
            )
        )?;

        // write commands
        stdout().flush()?;

        Ok(())
    }

    fn print_line(&self, s: &str) -> crossterm::Result<()> {
        let fill = " ".repeat(self.dimensions.0 - s.chars().count());
        queue!(
            stdout(),

            Print(s),
            Print(fill),
        )
    }

    fn content_to_string(&self) -> String {
        let mut s = String::new();
        for (idx, line) in self.content.iter().enumerate() {
            if idx > 0 { s.push('\n') }
            for c in line {
                s.push(*c);
            }
        }
        s
    }
}

fn main() -> anyhow::Result<()> { 
    // parse command line arguments
    let matches = App::new("terminal-editor")
       .about("A simple terminal text editor!")
       .arg(Arg::with_name("FILE")
            .help("Sets the input file to use")
            .required(true)
            .index(1))
       .get_matches();
    let path = matches.value_of("FILE").unwrap().to_string();

    // try to read file. If unsuccessful (e.g. file doesn't exist), use empty string.
    let text = std::fs::read_to_string(&path).ok();
    let content = match &text {
        Some(s) => {
            let mut res: Vec<Vec<char>> = s.lines().map(|x| x.chars().collect()).collect();
            match s.chars().last() {
                None | Some('\n') => res.push(vec!()),
                _ => {}
            }
            res
        },
        None => vec!(vec!())
    };

    // setup terminal
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    
    // create model
    let mut model = Model {
        exit: false,
        file_path: path,
        dimensions: crossterm::terminal::size().map(|(x,y)| (x as usize, y as usize))?,
        content: content,
        orig: text,
        cursor: (0, 0),
    };
    // render ui for the first time
    model.view()?;

    // main loop: read event, update model, view model, repeat
    while !model.exit {
        let evt = crossterm::event::read().expect("Reading event failed");
        model.update(evt);
        model.view()?;
    }

    // restore terminal state
    crossterm::terminal::disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    Ok(())
}
