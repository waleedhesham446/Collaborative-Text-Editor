use std::io::{self, stdout, stdin, Write, Read};
use std::fs;
use std::env;
use std::process::{Command, Stdio};

use warp::Filter;
use warp::ws::{WebSocket, Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use parking_lot::Mutex;

use simplelog::*;
use std::fs::File;
use log::debug;

static mut quit: bool = false;

fn init_logging() {
    let _ = WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").unwrap(),
    );
}

fn index_to_line_column(s: &str, index: usize) -> Option<(usize, usize)> {
    if index > s.len() {
        return None;
    }

    let mut line = 0;
    let mut col = 0;
    let mut current_index = 0;

    for c in s.chars() {
        if current_index == index {
            return Some((line, col));
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_index += c.len_utf8(); // account for UTF-8 character width
    }

    if index == current_index {
        // Edge case: index is at the end of the string
        return Some((line, col));
    }

    None
}

// Terminal handling
pub struct Terminal;

impl Terminal {
    pub fn enter_raw_mode() -> io::Result<()> {
        // Disable canonical mode, echo, and flow control (Ctrl+S/Ctrl+Q)
        Command::new("stty")
            .args(&["-icanon", "-echo", "-ixon"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        Ok(())
    }

    pub fn exit_raw_mode() -> io::Result<()> {
        // Re-enable canonical mode and echo
        Command::new("stty")
            .args(&["icanon", "echo"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        Ok(())
    }

    pub fn clear_screen() -> io::Result<()> {
        print!("\x1b[2J\x1b[H");
        stdout().flush()?;
        Ok(())
    }

    pub fn hide_cursor() -> io::Result<()> {
        print!("\x1b[?25l");
        stdout().flush()?;
        Ok(())
    }

    pub fn show_cursor() -> io::Result<()> {
        print!("\x1b[?25h");
        stdout().flush()?;
        Ok(())
    }

    pub fn move_cursor(row: usize, col: usize) -> io::Result<()> {
        print!("\x1b[{};{}H", row + 1, col + 1);
        stdout().flush()?;
        Ok(())
    }

    pub fn get_terminal_size() -> io::Result<(usize, usize)> {
        let output = Command::new("tput")
            .args(&["lines"])
            .output()?;
        let rows = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(24);

        let output = Command::new("tput")
            .args(&["cols"])
            .output()?;
        let cols = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(80);

        Ok((rows, cols))
    }

    pub fn read_key() -> io::Result<Key> {
        let mut buffer = [0; 1];
        stdin().read_exact(&mut buffer)?;
        
        match buffer[0] {
            b'\x1b' => {
                // Escape sequence - try to read more bytes
                let mut seq_buffer = [0; 2];
                match stdin().read_exact(&mut seq_buffer) {
                    Ok(_) => {
                        if seq_buffer[0] == b'[' {
                            match seq_buffer[1] {
                                b'A' => Ok(Key::Up),
                                b'B' => Ok(Key::Down),
                                b'C' => Ok(Key::Right),
                                b'D' => Ok(Key::Left),
                                b'H' => Ok(Key::Home),
                                b'F' => Ok(Key::End),
                                _ => Ok(Key::Escape),
                            }
                        } else {
                            Ok(Key::Escape)
                        }
                    }
                    Err(_) => Ok(Key::Escape),
                }
            }
            b'\r' | b'\n' => Ok(Key::Enter),
            b'\x7f' | b'\x08' => Ok(Key::Backspace),
            b'\x04' => Ok(Key::Delete),
            1 => Ok(Key::Ctrl(b'a')),   // Ctrl+A
            2 => Ok(Key::Ctrl(b'b')),   // Ctrl+B
            3 => Ok(Key::Ctrl(b'c')),   // Ctrl+C
            4 => Ok(Key::Ctrl(b'd')),   // Ctrl+D
            5 => Ok(Key::Ctrl(b'e')),   // Ctrl+E
            6 => Ok(Key::Ctrl(b'f')),   // Ctrl+F
            7 => Ok(Key::Ctrl(b'g')),   // Ctrl+G
            8 => Ok(Key::Ctrl(b'h')),   // Ctrl+H
            9 => Ok(Key::Ctrl(b'i')),   // Ctrl+I (Tab)
            10 => Ok(Key::Ctrl(b'j')),  // Ctrl+J
            11 => Ok(Key::Ctrl(b'k')),  // Ctrl+K
            12 => Ok(Key::Ctrl(b'l')),  // Ctrl+L
            13 => Ok(Key::Ctrl(b'm')),  // Ctrl+M
            14 => Ok(Key::Ctrl(b'n')),  // Ctrl+N
            15 => Ok(Key::Ctrl(b'o')),  // Ctrl+O
            16 => Ok(Key::Ctrl(b'p')),  // Ctrl+P
            17 => Ok(Key::Ctrl(b'q')),  // Ctrl+Q
            18 => Ok(Key::Ctrl(b'r')),  // Ctrl+R
            19 => Ok(Key::Ctrl(b's')),  // Ctrl+S
            20 => Ok(Key::Ctrl(b't')),  // Ctrl+T
            21 => Ok(Key::Ctrl(b'u')),  // Ctrl+U
            22 => Ok(Key::Ctrl(b'v')),  // Ctrl+V
            23 => Ok(Key::Ctrl(b'w')),  // Ctrl+W
            24 => Ok(Key::Ctrl(b'x')),  // Ctrl+X
            25 => Ok(Key::Ctrl(b'y')),  // Ctrl+Y
            26 => Ok(Key::Ctrl(b'z')),  // Ctrl+Z
            c if c >= 32 && c <= 126 => Ok(Key::Char(c as char)),
            c => Ok(Key::Ctrl(c)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Key {
    Char(char),
    Ctrl(u8),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Enter,
    Backspace,
    Delete,
    Escape,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct TextChange {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

// Type alias for the WebSocket sender
type WsSender = futures_util::stream::SplitSink<WebSocket, Message>;

// Editor implementation
pub struct Editor {
    content: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    offset_x: usize,
    offset_y: usize,
    terminal_rows: usize,
    terminal_cols: usize,
    filename: Option<String>,
    modified: bool,
    status_message: String,
    // Add WebSocket sender as an optional field
    ws_sender: Option<Arc<Mutex<WsSender>>>,
}

impl Editor {
    pub fn new(filename: Option<String>) -> io::Result<Self> {
        let (rows, cols) = Terminal::get_terminal_size()?;
        
        let content = if let Some(ref fname) = filename {
            match fs::read_to_string(fname) {
                Ok(content) => {
                    if content.is_empty() {
                        vec![String::new()]
                    } else {
                        content.lines().map(|s| s.to_string()).collect()
                    }
                }
                Err(_) => vec![String::new()],
            }
        } else {
            vec![String::new()]
        };

        Ok(Editor {
            content,
            cursor_x: 0,
            cursor_y: 0,
            offset_x: 0,
            offset_y: 0,
            terminal_rows: rows.saturating_sub(2), // Reserve space for status bar
            terminal_cols: cols,
            filename,
            modified: false,
            status_message: "Press Ctrl+Q to quit, Ctrl+S to save, Ctrl+H for help".to_string(),
            ws_sender: None,
        })
    }

    // Method to broadcast editor state changes
    pub async fn broadcast_change(&self, key: &Key, start: usize) -> Result<(), Box<dyn std::error::Error>> {
        let text = match key {
            Key::Char(c) => c.to_string(),
            Key::Enter => "\n".to_string(),
            Key::Backspace => "".to_string(), // Handle backspace separately
            _ => "".to_string(),
        };

        let mut start = start;
        if !text.is_empty() {
            start -= 1;
        }

        let change = TextChange {
            text,
            start,
            end: start + 1,
        };

        debug!("Broadcasting change: {:?}", change);
        
        if let Some(ref sender) = self.ws_sender {
            let message = serde_json::to_string(&change)?;
            let mut sender_guard = sender.lock();
            sender_guard.send(Message::text(message)).await?;
        }
        Ok(())
    }

    fn refresh_screen(&self) -> io::Result<()> {
        Terminal::clear_screen()?;
        // Draw content
        for row in 0..self.terminal_rows {
            let file_row = row + self.offset_y;
            if file_row < self.content.len() {
                let line = &self.content[file_row];
                let start = self.offset_x.min(line.len());
                let end = (self.offset_x + self.terminal_cols).min(line.len());
                if start < end {
                    print!("{}", &line[start..end]);
                }
            } else {
                print!("~");
            }
            print!("\r\n");
        }

        // Draw status bar
        let status = if let Some(ref filename) = self.filename {
            format!("{} - {} lines{}", 
                filename, 
                self.content.len(),
                if self.modified { " (modified)" } else { "" })
        } else {
            format!("[No Name] - {} lines{}", 
                self.content.len(),
                if self.modified { " (modified)" } else { "" })
        };
        
        print!("\x1b[7m{:<width$}\x1b[m\r\n", status, width = self.terminal_cols);
        print!("{}", self.status_message);

        // Position cursor
        Terminal::move_cursor(
            self.cursor_y - self.offset_y,
            self.cursor_x - self.offset_x
        )?;
        
        Ok(())
    }

    fn process_keypress(&mut self, key: &Key) -> io::Result<bool> {
        let changed: io::Result<bool> = match key {
            Key::Ctrl(b'q') => {
                if self.modified {
                    self.status_message = "File has unsaved changes! Press Ctrl+Q again to quit.".to_string();
                    self.modified = false; // Next Ctrl+Q will quit
                } else {
                    unsafe {
                        quit = true;
                    }
                }
                return Ok(false);
            }
            Key::Ctrl(b's') => {
                self.save_file()?;
                return Ok(false);
            }
            Key::Ctrl(b'h') => {
                self.status_message = "Ctrl+Q: Quit | Ctrl+S: Save | Arrow keys: Navigate | Enter: New line".to_string();
                return Ok(false);
            }
            Key::Up => self.move_cursor_up(),
            Key::Down => self.move_cursor_down(),
            Key::Left => self.move_cursor_left(),
            Key::Right => self.move_cursor_right(),
            Key::Home => {
                self.cursor_x = 0;
                return Ok(false);
            },
            Key::End => {
                if self.cursor_y < self.content.len() {
                    self.cursor_x = self.content[self.cursor_y].len();
                }
                return Ok(false);
            }
            Key::Enter => self.insert_newline(),
            Key::Backspace => self.delete_char(),
            Key::Delete => self.delete_char_forward(),
            Key::Char(c) => self.insert_char(*c),
            _ => {
                // Debug: show what key was pressed
                if let Key::Ctrl(code) = key {
                    self.status_message = format!("Pressed Ctrl+{} (code: {})", (code + b'a' - 1) as char, code);
                }
                return Ok(false);
            }
        };

        self.scroll();
        changed
    }

    fn move_cursor_up(&mut self) -> io::Result<bool> {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = self.content[self.cursor_y].len();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
        }
        return Ok(false);
    }

    fn move_cursor_down(&mut self) -> io::Result<bool> {
        if self.cursor_y < self.content.len() - 1 {
            self.cursor_y += 1;
            let line_len = self.content[self.cursor_y].len();
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
        }
        return Ok(false);
    }

    fn move_cursor_left(&mut self) -> io::Result<bool> {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.content[self.cursor_y].len();
        }
        return Ok(false);
    }

    fn move_cursor_right(&mut self) -> io::Result<bool> {
        if self.cursor_y < self.content.len() {
            let line_len = self.content[self.cursor_y].len();
            if self.cursor_x < line_len {
                self.cursor_x += 1;
            } else if self.cursor_y < self.content.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
        }
        return Ok(false);
    }

    fn insert_char(&mut self, c: char) -> io::Result<bool> {
        if self.cursor_y == self.content.len() {
            self.content.push(String::new());
        }
        
        self.content[self.cursor_y].insert(self.cursor_x, c);
        self.cursor_x += 1;
        self.modified = true;
        self.status_message.clear();

        return Ok(true);
    }

    fn insert_newline(&mut self) -> io::Result<bool> {
        if self.cursor_y == self.content.len() {
            self.content.push(String::new());
        }
        
        let current_line = &self.content[self.cursor_y];
        let new_line = current_line[self.cursor_x..].to_string();
        self.content[self.cursor_y] = current_line[..self.cursor_x].to_string();
        
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.content.insert(self.cursor_y, new_line);
        self.modified = true;
        self.status_message.clear();
        
        return Ok(true);
    }

    fn delete_char(&mut self) -> io::Result<bool> {
        if self.cursor_x == 0 && self.cursor_y == 0 {
            return Ok(false);
        }

        if self.cursor_x > 0 {
            self.content[self.cursor_y].remove(self.cursor_x - 1);
            self.cursor_x -= 1;
        } else {
            let current_line = self.content.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.content[self.cursor_y].len();
            self.content[self.cursor_y].push_str(&current_line);
        }
        
        self.modified = true;
        self.status_message.clear();

        return Ok(true);
    }

    fn delete_char_forward(&mut self) -> io::Result<bool> {
        if self.cursor_y >= self.content.len() {
            return Ok(false);
        }

        let line_len = self.content[self.cursor_y].len();
        if self.cursor_x < line_len {
            self.content[self.cursor_y].remove(self.cursor_x);
        } else if self.cursor_y < self.content.len() - 1 {
            let next_line = self.content.remove(self.cursor_y + 1);
            self.content[self.cursor_y].push_str(&next_line);
        }
        
        self.modified = true;
        self.status_message.clear();

        return Ok(true);
    }

    fn scroll(&mut self) {
        if self.cursor_y < self.offset_y {
            self.offset_y = self.cursor_y;
        }
        if self.cursor_y >= self.offset_y + self.terminal_rows {
            self.offset_y = self.cursor_y - self.terminal_rows + 1;
        }
        if self.cursor_x < self.offset_x {
            self.offset_x = self.cursor_x;
        }
        if self.cursor_x >= self.offset_x + self.terminal_cols {
            self.offset_x = self.cursor_x - self.terminal_cols + 1;
        }
    }

    fn save_file(&mut self) -> io::Result<()> {
        let filename = if let Some(ref fname) = self.filename {
            fname.clone()
        } else {
            self.status_message = "Enter filename: ".to_string();
            return Ok(()); // For now, just show message
        };

        let content = self.content.join("\n");
        match fs::write(&filename, content) {
            Ok(_) => {
                self.modified = false;
                self.status_message = format!("Saved to {}", filename);
            }
            Err(e) => {
                self.status_message = format!("Error saving: {}", e);
            }
        }
        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::exit_raw_mode();
        let _ = Terminal::show_cursor();
    }
}

pub async fn handle_connection(ws: WebSocket, buffer: Arc<Mutex<Editor>>) {
    let (sender, mut receiver) = ws.split();
    
    // Store the sender in the editor
    {
        let mut editor = buffer.lock();
        editor.ws_sender = Some(Arc::new(Mutex::new(sender)));
    }

    while let Some(Ok(msg)) = receiver.next().await {
        if msg.is_text() {
            if let Ok(change) = serde_json::from_str::<TextChange>(msg.to_str().unwrap()) {
                {
                    debug!("Applying change: {:?}", change);
                    
                    let mut editor = buffer.lock();

                    if change.text.is_empty() {
                        if let Some((line, col)) = index_to_line_column(&editor.content.join("\n"), change.start) {
                            let editor_lines = editor.content.len();
                            if line < editor_lines {
                                let line_content = &mut editor.content[line];
                                if col < line_content.len() {
                                    line_content.remove(col);
                                } else if col == line_content.len() && line < editor_lines - 1 {
                                    let next_line = editor.content.remove(line + 1);
                                    editor.content[line].push_str(&next_line);
                                }
                            }
                        }
                    } else {
                        if let Some((line, col)) = index_to_line_column(&editor.content.join("\n"), change.start) {
                            // Handle newline insertion
                            if (change.text == "\n" || change.text == "\r\n") && line < editor.content.len() {
                                let line_content = &mut editor.content[line];
                                let new_line = line_content[col..].to_string();
                                line_content.truncate(col);
                                editor.content.insert(line + 1, new_line);
                                editor.cursor_y += 1;
                                editor.cursor_x = 0;
                            } else {
                                if line < editor.content.len() {
                                    let line_content = &mut editor.content[line];
                                    line_content.insert_str(col, &change.text);
                                    editor.cursor_x += change.text.len();
                                }
                            }
                        }
                    }
                    
                    editor.modified = true;
                    editor.status_message = format!("Applied change: {:?}", change);
                    editor.refresh_screen().unwrap();
                }
            }
        }
    }
}

pub async fn run(buffer: Arc<Mutex<Editor>>) -> io::Result<()> {
    Terminal::enter_raw_mode()?;
    Terminal::clear_screen()?;
    
    unsafe {
        while !quit {
            let editor = buffer.lock();
            editor.refresh_screen()?;
            std::mem::drop(editor);

            let key = Terminal::read_key()?;
            debug!("Key pressed: {:?}", key);
            
            let mut editor2 = buffer.lock();
            let changed = editor2.process_keypress(&key)?;

            if changed {
                let start = editor2.content.iter().take(editor2.cursor_y).map(|s| s.len() + 1).sum::<usize>() + editor2.cursor_x;
                if let Err(e) = editor2.broadcast_change(&key, start).await {
                    eprintln!("Error broadcasting change: {}", e);
                }
            }

            std::mem::drop(editor2);
        }
    }

    Terminal::exit_raw_mode()?;
    Terminal::show_cursor()?;
    Terminal::clear_screen()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    let buffer = Arc::<Mutex::<Editor>>::new(Mutex::new(Editor::new(filename)?));

    init_logging();

    // Clone for the warp server
    let buffer_for_ws = buffer.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || buffer_for_ws.clone()))
        .map(|ws: warp::ws::Ws, buffer| {
            ws.on_upgrade(move |socket| handle_connection(socket, buffer))
        });

    // Run WebSocket server in the background
    tokio::spawn(async move {
        warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
    });

    run(buffer)
        .await
        .expect("Failed to run editor");
    
    Ok(())
}