use std::io::{self, Stdout, Write};

use crossterm::{
    cursor, execute, queue,
    style::{self, Print, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::color::Color;

/// A cell in the frame buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub color: Color,
}

/// 2D grid of rendered cells.
pub struct FrameBuffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Vec<Option<Cell>>>,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        FrameBuffer {
            width,
            height,
            cells: vec![vec![None; width as usize]; height as usize],
        }
    }

    pub fn set(&mut self, x: i32, y: i32, ch: char, color: Color) {
        if x >= 0 && y >= 0 && (x as u16) < self.width && (y as u16) < self.height {
            self.cells[y as usize][x as usize] = Some(Cell { ch, color });
        }
    }
}

pub struct Renderer {
    stdout: Stdout,
    prev_frame: Option<Vec<Vec<Option<Cell>>>>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            stdout: io::stdout(),
            prev_frame: None,
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            self.stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All)
        )?;
        Ok(())
    }

    pub fn draw(&mut self, frame: &FrameBuffer) -> io::Result<()> {
        let mut last_color: Option<Color> = None;

        for y in 0..frame.height {
            for x in 0..frame.width {
                let cell = frame.cells[y as usize][x as usize];
                let prev = self
                    .prev_frame
                    .as_ref()
                    .and_then(|pf| pf.get(y as usize)?.get(x as usize).copied())
                    .flatten();

                if cell == prev {
                    continue;
                }

                queue!(self.stdout, cursor::MoveTo(x, y))?;

                match cell {
                    Some(c) => {
                        if last_color != Some(c.color) {
                            queue!(
                                self.stdout,
                                SetForegroundColor(c.color.to_crossterm())
                            )?;
                            last_color = Some(c.color);
                        }
                        queue!(self.stdout, Print(c.ch))?;
                    }
                    None => {
                        queue!(self.stdout, Print(' '))?;
                    }
                }
            }
        }

        self.stdout.flush()?;
        self.prev_frame = Some(frame.cells.clone());
        Ok(())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.prev_frame = None;
        execute!(self.stdout, terminal::Clear(ClearType::All))
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}
