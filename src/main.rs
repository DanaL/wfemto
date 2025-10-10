// wpico - A toy text editor
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

/// Represents the text editor state
struct TextEditor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
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
            cursor_x: 0,
            cursor_y: 0,
            prev_cursor_x: 0,
            prev_cursor_y: 0,
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
            let pos = self.cursor_x - OPEN_FILE_MARGIN;
            self.input_buffer.insert(pos, c);
            self.cursor_x += 1;
        } else {
            let line = &mut self.lines[self.cursor_y];
            line.insert(self.cursor_x, c);
            self.cursor_x += 1;
            self.is_modified = true;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let line = &mut self.lines[self.cursor_y];
            line.remove(self.cursor_x - 1);
            self.cursor_x -= 1;
            self.is_modified = true;
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
            self.lines[self.cursor_y].push_str(&current_line);
            self.is_modified = true;
        }
    }

    fn backspace_buffer(&mut self, offset: usize) {
        if self.input_buffer.is_empty() || self.cursor_x == offset {
            return;
        }

        let buffer_pos = self.cursor_x - offset - 1;
        if buffer_pos <= self.input_buffer.len() {
            self.input_buffer.remove(buffer_pos );
            self.cursor_x -= 1;
        }
    }

    fn insert_newline(&mut self) {
        let current_line = &mut self.lines[self.cursor_y];

        // Split line at cursor
        let rest_of_line = current_line[self.cursor_x..].to_string();

        self.lines[self.cursor_y].truncate(self.cursor_x);

        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, rest_of_line);
        self.cursor_x = 0;
        self.is_modified = true;
    }

    fn move_cursor_left(&mut self) {
        if self.mode == EditorMode::OpenFile {
            if self.cursor_x - OPEN_FILE_MARGIN > 0 {
                self.cursor_x -= 1;
            }
            
            return;
        }

        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();            
        }
    }
    
    fn move_cursor_right(&mut self) {
        if self.mode == EditorMode::OpenFile {
            if self.cursor_x < self.input_buffer.len() + OPEN_FILE_MARGIN {
                self.cursor_x += 1;
            }
            return;
        } 

        if self.cursor_x < self.lines[self.cursor_y].len() {
            self.cursor_x += 1;
        } else if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;

            if self.cursor_x > self.lines[self.cursor_y].len() {
                self.cursor_x = self.lines[self.cursor_y].len();
            }
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;

            if self.cursor_x > self.lines[self.cursor_y].len() {
                self.cursor_x = self.lines[self.cursor_y].len();
            }
        }
    }

    /// Save the current file
    fn save(&mut self) {
        // TODO: Implement file saving
        println!("Save file: {}", self.filename);
    }

    /// Load a file
    fn load(&mut self, filename: &str) -> Result<(), String> {
        let file = File::open(filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        self.lines.clear();
        for line in reader.lines() {
            self.lines.push(line.map_err(|e| e.to_string())?);
        }

        self.filename = filename.to_string();
        self.cursor_x = 0;
        self.cursor_y = 0;
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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::TextInput { text, .. } => {
                    for c in text.chars() {
                        editor.insert_char(c);
                    }
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    keymod,
                    ..
                } => {
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
                        Keycode::Right => editor.move_cursor_right(),
                        Keycode::Up => {
                            if editor.mode == EditorMode::Edit {
                                editor.move_cursor_up()
                            }
                        },
                        Keycode::Down => {
                            if editor.mode == EditorMode::Edit {
                                editor.move_cursor_down()
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
                                editor.prev_cursor_x = editor.cursor_x;
                                editor.prev_cursor_y = editor.cursor_y;
                                editor.cursor_x = OPEN_FILE_MARGIN;
                                editor.cursor_y = EDITOR_ROWS as usize;
                            }
                        },
                        Keycode::Home => {
                            if editor.mode == EditorMode::Edit {
                                editor.cursor_x = 0;
                            }
                            else {
                                editor.cursor_x = OPEN_FILE_MARGIN;
                            }
                        },
                        Keycode::End => {
                            if editor.mode == EditorMode::Edit {
                                editor.cursor_x = editor.lines[editor.cursor_y].len();
                            }
                            else {
                                editor.cursor_x = editor.input_buffer.len() + OPEN_FILE_MARGIN;
                            }
                        },
                        Keycode::Escape => { 
                            editor.mode = EditorMode::Edit;
                            editor.cursor_x = editor.prev_cursor_x;
                            editor.cursor_y = editor.prev_cursor_y;
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

        for (i, line) in editor.lines.iter().enumerate() {
            render_text(
                &mut canvas,
                &font,
                line,
                10, 10 + (i as i32 * window_info.char_height as i32), Color::RGB(0, 0, 0))?;
        }
        
        if editor.last_cursor_blink.elapsed() >= Duration::from_millis(500) {
            editor.cursor_visible = !editor.cursor_visible;
            editor.last_cursor_blink = std::time::Instant::now();
        }
        
        draw_status_bar(&mut canvas, &font, &editor, &window_info)?;
        
        if editor.cursor_visible {
            canvas.set_draw_color(Color::RGB(128, 128, 128));
            
            // Calculate actual text width up to cursor position
            // NB: char_width * text was inaccute
            let text_width = if editor.mode == EditorMode::Edit {
                let text_before_cursor = &editor.lines[editor.cursor_y][..editor.cursor_x];
                font.size_of(text_before_cursor).unwrap_or((0, 0)).0
            } else {
                let status = format!("Open file: {}", &editor.input_buffer[..editor.cursor_x - OPEN_FILE_MARGIN]);
                font.size_of(&status).unwrap_or((0, 0)).0
            };
            
            let cursor_rect = Rect::new(
                MARGIN_LEFT + text_width as i32,
                MARGIN_TOP + (editor.cursor_y as i32 * window_info.char_height as i32),
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
