// wfemto - A toy text editor
// Written in 2025 by Dana Larose <ywg.dana@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all copyright
// and related and neighboring rights to this software to the public domain
// worldwide. This software is distributed without any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication along 
// with this software. If not, 
// see <http://creativecommons.org/publicdomain/zero/1.0/>.

extern crate sdl2;

use std::cmp;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

const EDITOR_COLS: u32 = 80;
const EDITOR_ROWS: u32 = 32;
const FONT_SIZE: u16 = 14;
const MARGIN_LEFT: i32 = 10;
const MARGIN_TOP: i32 = 10;

const OPEN_FILE_MARGIN: usize = 11;

#[derive(PartialEq)]
enum EditorMode {
    Edit,
    OpenFile
}

struct WindowInfo {
    rows: u32,
    cols: u32,
    char_width: u32,
    char_height: u32,    
}

struct TextEditor {
    lines: Vec<String>,
    scr_col: usize,
    scr_row: usize,
    buffer_col: usize,
    buffer_row: usize,
    prev_cursor_x: usize,
    prev_cursor_y: usize,
    filename: String,
    is_modified: bool,
    cursor_visible: bool,
    last_cursor_blink: std::time::Instant,
    mode: EditorMode,
    input_buffer: String,  // Buffer for command/filename input    
}

impl TextEditor {
    fn new() -> Self {
        TextEditor {
            lines: vec![String::new()],
            scr_col: 0,
            scr_row: 0,
            prev_cursor_x: 0,
            prev_cursor_y: 0,
            buffer_col: 0,
            buffer_row: 0,
            filename: String::from("filename.txt"),
            is_modified: false,
            cursor_visible: true,
            last_cursor_blink: std::time::Instant::now(),
            mode: EditorMode::Edit,
            input_buffer: String::new(),
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.mode == EditorMode::OpenFile {
            let pos = self.scr_col - OPEN_FILE_MARGIN;
            self.input_buffer.insert(pos, c);
            self.scr_col += 1;
        } else {
            let line = &mut self.lines[self.buffer_row];
            line.insert(self.scr_col, c);
            self.scr_col += 1;
            self.is_modified = true;
        }
    }

    fn backspace(&mut self) {
        if self.scr_col > 0 {
            let line = &mut self.lines[self.buffer_row];
            line.remove(self.scr_col - 1);
            self.scr_col -= 1;
            self.is_modified = true;
        } else if self.buffer_row > 0 {
            let current_line = self.lines.remove(self.buffer_row);
            self.buffer_row -= 1;
            self.scr_col = self.lines[self.buffer_row].len();
            self.lines[self.buffer_row].push_str(&current_line);
            self.is_modified = true;
        }
    }

    fn backspace_buffer(&mut self, offset: usize) {
        if self.input_buffer.is_empty() || self.scr_col == offset {
            return;
        }

        let buffer_pos = self.scr_col - offset - 1;
        if buffer_pos <= self.input_buffer.len() {
            self.input_buffer.remove(buffer_pos );
            self.scr_col -= 1;
        }
    }

    fn insert_newline(&mut self) {
        let current_line = &mut self.lines[self.buffer_row];

        // Split line at cursor
        let rest_of_line = current_line[self.scr_col..].to_string();

        self.lines[self.buffer_row].truncate(self.scr_col);

        self.buffer_row += 1;
        self.scr_row = cmp::min(self.scr_row + 1, EDITOR_ROWS as usize - 1);
        self.lines.insert(self.buffer_row, rest_of_line);
        self.scr_col = 0;
        self.is_modified = true;
    }

    fn move_cursor_left(&mut self) {
        if self.mode == EditorMode::OpenFile {
            if self.scr_col - OPEN_FILE_MARGIN > 0 {
                self.scr_col -= 1;
            }
            
            return;
        }

        if self.scr_col > 0 {
            self.scr_col -= 1;
        } else if self.scr_row > 0 {
            self.buffer_row -= 1;
            self.scr_col = self.lines[self.buffer_row].len();            
        }
    }
    
    fn move_cursor_right(&mut self, window_info: &WindowInfo) {
        if self.mode == EditorMode::OpenFile {
            if self.scr_col < self.input_buffer.len() + OPEN_FILE_MARGIN {
                self.scr_col += 1;
            }
            return;
        } 

        println!("{}", self.lines[self.buffer_row].len());
        if self.buffer_col < self.lines[self.buffer_row].len() {
            self.buffer_col += 1;

            if self.scr_col < window_info.cols as usize - 1 {
                println!("{} {} {}", window_info.cols, self.buffer_col, self.scr_col);
                self.scr_col += 1;
            }
        } else if self.buffer_row < self.lines.len() - 1 {
            self.buffer_row += 1;
            self.buffer_col = 0;
            self.scr_col = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.buffer_row > 0 {
            self.buffer_row -= 1;

            if self.scr_col > self.lines[self.buffer_row].len() {
                self.scr_col = self.lines[self.buffer_row].len();
            }
        }

        if self.scr_row > 0 && !(self.scr_row == 5 && self.buffer_row > 5) {
            self.scr_row -= 1;
        }
    }
    
    fn move_cursor_down(&mut self, window_info: &WindowInfo) {
        if self.buffer_row == self.lines.len() - 1 {
            return
        }
        
        if self.buffer_row < self.lines.len() - 1 {
            self.buffer_row += 1;

            if self.scr_col > self.lines[self.buffer_row].len() {
                self.scr_col = self.lines[self.buffer_row].len();
            }
        }

        let bm = EDITOR_ROWS as usize - 5;
        if self.scr_row < window_info.rows as usize - 1 && !(self.scr_row == bm && self.buffer_row < self.lines.len() - 5) {
            self.scr_row += 1;
        }
    }

    /// Save the current file
    fn save(&mut self) {
        // TODO: Implement file saving
        println!("Save file: {}", self.filename);
    }

    fn load(&mut self, filename: &str) -> Result<(), String> {
        let file = File::open(filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        self.lines.clear();
        for line in reader.lines() {
            self.lines.push(line.map_err(|e| e.to_string())?);
        }

        self.filename = filename.to_string();
        self.scr_col = 0;
        self.scr_row = 0;
        self.buffer_row = 0;
        self.is_modified = false;

        Ok(())
    }
}

fn render_text(
    canvas: &mut Canvas<Window>,
    font: &Font,
    text: &str,
    x: i32,
    y: i32,
    colour: Color,
) -> Result<(), String> {
    
    if text.is_empty() {
        return Ok(());
    }
    
    let surface = font
        .render(text).blended(colour).map_err(|e| e.to_string())?;
    
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    
    let target = Rect::new(x, y, surface.width(), surface.height());
    canvas.copy(&texture, None, Some(target))?;
    
    Ok(())
}

fn draw_status_bar(
    canvas: &mut Canvas<Window>, 
    font: &Font, 
    editor: &TextEditor, 
    window_info: &WindowInfo
) -> Result<(), String> {    
    let status = match editor.mode {
        EditorMode::Edit => { 
            let mut status = editor.filename.clone();  
            if editor.is_modified {
                status.push('*');
            }
            status
        },
        EditorMode::OpenFile => {
            let mut status = String::from("Open file: ");
            status.push_str(&editor.input_buffer);
            status
        },
    };
    
    let status_bar_row_pixels = window_info.rows * window_info.char_height + MARGIN_TOP as u32;

    canvas.set_draw_color(Color::RGB(217, 217, 214));
    canvas.fill_rect(Rect::new(0, status_bar_row_pixels as i32, 
        window_info.cols * window_info.char_width + (MARGIN_LEFT as u32 * 2), window_info.char_height)).map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    render_text(
        canvas,
        font,
        &status,
        10, status_bar_row_pixels as i32, Color::RGB(89, 89, 88))?;

    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path = "DejaVuSansMono.ttf";    
    let font = ttf_context.load_font(font_path, FONT_SIZE)?;
    
    let (char_width, char_height) = font.size_of("X").map_err(|e| e.to_string())?;

    let window_width = EDITOR_COLS * char_width + (MARGIN_LEFT * 2) as u32;
    let window_height = ((EDITOR_ROWS + 1) * char_height) + MARGIN_TOP as u32;

    let window_info = WindowInfo { rows: EDITOR_ROWS, cols: EDITOR_COLS, char_width, char_height };

    let window = video_subsystem
        .window("wfemto", window_width, window_height)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut editor = TextEditor::new();
    let mut event_pump = sdl_context.event_pump()?;

    let mut splash_title= true;
    
    //editor.load("src/main.rs")?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::TextInput { text, .. } => {
                    for c in text.chars() {
                        editor.insert_char(c);
                    }
                    splash_title= false;
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
                    splash_title= false;
                    // Handle special keys
                    match keycode {
                        Keycode::Return => if editor.mode == EditorMode::Edit {
                            editor.insert_newline()
                        } else {
                            let filename = editor.input_buffer.clone();
                            editor.load(&filename);
                            editor.mode = EditorMode::Edit;                            
                        },
                        Keycode::Backspace => {
                            if editor.mode == EditorMode::Edit {
                                editor.backspace();
                            } else {
                                editor.backspace_buffer(OPEN_FILE_MARGIN);
                            }
                        },
                        Keycode::Left => editor.move_cursor_left(),
                        Keycode::Right => editor.move_cursor_right(&window_info),
                        Keycode::Up => {
                            if editor.mode == EditorMode::Edit {
                                editor.move_cursor_up()
                            }
                        },
                        Keycode::Down => {
                            if editor.mode == EditorMode::Edit {
                                editor.move_cursor_down(&window_info)
                            }                            
                        },
                        Keycode::Q if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
                            || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD) =>
                        {
                            break 'running;
                        }
                        Keycode::S if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
                            || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD) =>
                        {
                            editor.save();
                        },
                        Keycode::O if keymod.contains(sdl2::keyboard::Mod::LCTRLMOD)
                            || keymod.contains(sdl2::keyboard::Mod::RCTRLMOD) =>
                        {
                            if editor.mode != EditorMode::OpenFile {
                                editor.mode = EditorMode::OpenFile;
                                editor.input_buffer = String::new();
                                editor.prev_cursor_x = editor.scr_col;
                                editor.prev_cursor_y = editor.scr_row;
                                editor.scr_col = OPEN_FILE_MARGIN;
                                editor.scr_row = EDITOR_ROWS as usize;
                            }
                        },
                        Keycode::Home => {
                            if editor.mode == EditorMode::Edit {
                                editor.scr_col = 0;
                            }
                            else {
                                editor.scr_col = OPEN_FILE_MARGIN;
                            }
                        },
                        Keycode::End => {
                            if editor.mode == EditorMode::Edit {
                                editor.scr_col = editor.lines[editor.buffer_row].len();
                            }
                            else {
                                editor.scr_col = editor.input_buffer.len() + OPEN_FILE_MARGIN;
                            }
                        },
                        Keycode::Escape => { 
                            editor.mode = EditorMode::Edit;
                            editor.scr_col = editor.prev_cursor_x;
                            editor.scr_row = editor.prev_cursor_y;
                        },
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        // Clear screen
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        if editor.lines.len() > 0 && splash_title {
            let s = String::from("wfemto 0.0.1 -- a toy text editor");
            let col = EDITOR_COLS as i32 / 2 - s.len() as i32 / 2;

            render_text(
                &mut canvas,
                &font,
                &s,
                col * window_info.char_width as i32, 
                MARGIN_TOP + (EDITOR_ROWS as i32 / 4 * window_info.char_height as i32), 
                Color::RGB(0, 0, 0))?;
            canvas.present();

            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS

            continue           
        } 
        
        let buffer_start = (editor.buffer_row as i32 - editor.scr_row as i32).max(0) as usize;
        let buffer_end = (buffer_start + window_info.rows as usize).min(editor.lines.len());
        
        let mut scr_row = 0;
        for buffer_row in buffer_start..buffer_end {                                
            let line = &editor.lines[buffer_row];                
            render_text(
                &mut canvas,
                &font,
                line,
                MARGIN_LEFT, 
                MARGIN_TOP + (scr_row as i32 * window_info.char_height as i32), 
                Color::RGB(0, 0, 0))?;
            scr_row += 1;
        }
        
        if editor.last_cursor_blink.elapsed() >= Duration::from_millis(500) {
            editor.cursor_visible = !editor.cursor_visible;
            editor.last_cursor_blink = std::time::Instant::now();
        }
        
        draw_status_bar(&mut canvas, &font, &editor, &window_info)?;
        
        if editor.cursor_visible {            
            canvas.set_draw_color(Color::RGB(128, 128, 128));
            
            // Calculate actual text width up to cursor position
            // NB: char_width * text was inaccurate
            let text_width = if editor.mode == EditorMode::OpenFile {
                let status = format!("Open file: {}", &editor.input_buffer[..editor.scr_col - OPEN_FILE_MARGIN]);
                font.size_of(&status).unwrap_or((0, 0)).0
            } else {
                let text_before_cursor = &editor.lines[editor.buffer_row][..editor.buffer_col];
                font.size_of(text_before_cursor).unwrap_or((0, 0)).0
            };
                        
            let cursor_rect = Rect::new(
                MARGIN_LEFT + text_width as i32,
                MARGIN_TOP + (editor.scr_row as i32 * window_info.char_height as i32),
                2,
                window_info.char_height,
            );
            canvas.fill_rect(cursor_rect).map_err(|e| e.to_string())?;
        }

        canvas.present();

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}
