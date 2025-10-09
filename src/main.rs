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

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use std::time::Duration;

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const FONT_SIZE: u16 = 16;

/// Represents the text editor state
struct TextEditor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    filename: String,
    is_modified: bool,
    cursor_visible: bool,
    last_cursor_blink: std::time::Instant,
}

impl TextEditor {
    fn new() -> Self {
        TextEditor {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            filename: String::from("filename.txt"),
            is_modified: false,
            cursor_visible: true,
            last_cursor_blink: std::time::Instant::now(),
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_y];
        line.insert(self.cursor_x, c);
        self.cursor_x += 1;
        self.is_modified = true;        
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
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
    }

    fn move_cursor_right(&mut self) {
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
    fn load(&mut self, filename: &str) {
        // TODO: Implement file loading
        println!("Load file: {}", filename);
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

fn draw_status_bar(canvas: &mut Canvas<Window>, font: &Font, editor: &TextEditor) -> Result<(), String> {
    let mut status = editor.filename.clone();
    if editor.is_modified {
        status.push_str("*");
    }
    
    canvas.set_draw_color(Color::RGB(217, 217, 214));
    canvas.fill_rect(Rect::new(0, (WINDOW_HEIGHT - 25) as i32, WINDOW_WIDTH, 25)).map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    render_text(
        canvas,
        &font,
        &status,
        10, (WINDOW_HEIGHT - 23) as i32, Color::RGB(89, 89, 88))?;

    Ok(())
}

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // Create window
    let window = video_subsystem
        .window("wpico 0.0.1", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path = "DejaVuSansMono.ttf";    
    let font = ttf_context.load_font(font_path, FONT_SIZE)?;
    
    let mut editor = TextEditor::new();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::TextInput { text, .. } => {
                    // Handle text input
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
                        Keycode::Return => editor.insert_newline(),
                        Keycode::Backspace => editor.backspace(),
                        Keycode::Left => editor.move_cursor_left(),
                        Keycode::Right => editor.move_cursor_right(),
                        Keycode::Up => editor.move_cursor_up(),
                        Keycode::Down => editor.move_cursor_down(),
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
                        Keycode::Home => editor.cursor_x = 0,
                        Keycode::End => editor.cursor_x = editor.lines[editor.cursor_y].len(),
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
                10, 10 + (i as i32 * 25), Color::RGB(0, 0, 0))?;
        }
        
        if editor.last_cursor_blink.elapsed() >= Duration::from_millis(500) {
            editor.cursor_visible = !editor.cursor_visible;
            editor.last_cursor_blink = std::time::Instant::now();
        }
        
        if editor.cursor_visible {
            canvas.set_draw_color(Color::RGB(128, 128, 128));
            let cursor_rect = Rect::new(
                10 + (editor.cursor_x as i32 * 10),
                10 + (editor.cursor_y as i32 * 25),
                2,
                16,
            );
            canvas.fill_rect(cursor_rect).map_err(|e| e.to_string())?;
        }
        
        draw_status_bar(&mut canvas, &font, &editor)?;

        canvas.present();

        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    Ok(())
}
