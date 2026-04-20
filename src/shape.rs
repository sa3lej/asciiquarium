use crate::color::Color;

/// A single frame of ASCII art with per-character color information.
#[derive(Clone, Debug)]
pub struct Frame {
    /// Each cell is Some(char) for visible, None for transparent.
    pub cells: Vec<Vec<Option<char>>>,
    /// Per-cell color, same dimensions as cells. None = use entity default_color.
    pub colors: Vec<Vec<Option<Color>>>,
    pub width: u16,
    pub height: u16,
}

impl Frame {
    /// Parse ASCII art string and optional color mask string into a Frame.
    /// `auto_trans` specifies characters to treat as transparent (typically '?' or ' ').
    pub fn parse(art: &str, mask: Option<&str>, auto_trans: Option<char>) -> Self {
        let art_lines: Vec<&str> = art.lines().collect();
        let mask_lines: Vec<&str> = mask.map(|m| m.lines().collect()).unwrap_or_default();

        let height = art_lines.len();
        let width = art_lines.iter().map(|l| l.len()).max().unwrap_or(0);

        let mut cells = Vec::with_capacity(height);
        let mut colors = Vec::with_capacity(height);

        for (row, art_line) in art_lines.iter().enumerate() {
            let mut cell_row = Vec::with_capacity(width);
            let mut color_row = Vec::with_capacity(width);

            let art_chars: Vec<char> = art_line.chars().collect();
            let mask_chars: Vec<char> = mask_lines
                .get(row)
                .map(|l| l.chars().collect())
                .unwrap_or_default();

            for col in 0..width {
                let ch = art_chars.get(col).copied();
                let is_transparent = match ch {
                    None => true,
                    Some(c) => auto_trans.map_or(false, |t| c == t),
                };

                if is_transparent {
                    cell_row.push(None);
                    color_row.push(None);
                } else {
                    cell_row.push(ch);
                    let color = mask_chars
                        .get(col)
                        .and_then(|&mc| Color::from_mask_char(mc));
                    color_row.push(color);
                }
            }

            cells.push(cell_row);
            colors.push(color_row);
        }

        Frame {
            cells,
            colors,
            width: width as u16,
            height: height as u16,
        }
    }
}

/// A shape is one or more frames for animation.
#[derive(Clone, Debug)]
pub struct Shape {
    pub frames: Vec<Frame>,
}

impl Shape {
    pub fn single(art: &str, mask: Option<&str>, auto_trans: Option<char>) -> Self {
        Shape {
            frames: vec![Frame::parse(art, mask, auto_trans)],
        }
    }

    pub fn multi(frames: Vec<(&str, Option<&str>)>, auto_trans: Option<char>) -> Self {
        Shape {
            frames: frames
                .into_iter()
                .map(|(art, mask)| Frame::parse(art, mask, auto_trans))
                .collect(),
        }
    }
}
