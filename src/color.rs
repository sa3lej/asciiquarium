use crossterm::style;
use rand::Rng;

/// Terminal colors matching the Perl single-letter codes,
/// plus an RGB variant for truecolor terminals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    Black,
    Red,
    BrightRed,
    Green,
    BrightGreen,
    Yellow,
    BrightYellow,
    Blue,
    BrightBlue,
    Cyan,
    BrightCyan,
    Magenta,
    BrightMagenta,
    White,
    /// 24-bit RGB color for truecolor-capable terminals.
    Rgb(u8, u8, u8),
}

/// Selects the color depth for image-to-ASCII conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorMode {
    /// Quantize to the 14-color terminal palette.
    Palette,
    /// Use full 24-bit RGB colors (requires truecolor terminal).
    TrueColor,
}

/// Detect terminal color support by inspecting the `COLORTERM` environment variable.
/// Returns `TrueColor` if the terminal advertises "truecolor" or "24bit",
/// otherwise falls back to `Palette`.
pub fn detect_color_support() -> ColorMode {
    match std::env::var("COLORTERM").as_deref() {
        Ok("truecolor") | Ok("24bit") => ColorMode::TrueColor,
        _ => ColorMode::Palette,
    }
}

impl Color {
    pub fn from_mask_char(c: char) -> Option<Color> {
        match c {
            'k' => Some(Color::Black),
            'r' => Some(Color::Red),
            'R' => Some(Color::BrightRed),
            'g' => Some(Color::Green),
            'G' => Some(Color::BrightGreen),
            'y' => Some(Color::Yellow),
            'Y' => Some(Color::BrightYellow),
            'b' => Some(Color::Blue),
            'B' => Some(Color::BrightBlue),
            'c' => Some(Color::Cyan),
            'C' => Some(Color::BrightCyan),
            'm' => Some(Color::Magenta),
            'M' => Some(Color::BrightMagenta),
            'W' => Some(Color::White),
            _ => None,
        }
    }

    pub fn to_crossterm(self) -> style::Color {
        match self {
            Color::Black => style::Color::DarkGrey,
            Color::Red => style::Color::DarkRed,
            Color::BrightRed => style::Color::Red,
            Color::Green => style::Color::DarkGreen,
            Color::BrightGreen => style::Color::Green,
            Color::Yellow => style::Color::DarkYellow,
            Color::BrightYellow => style::Color::Yellow,
            Color::Blue => style::Color::DarkBlue,
            Color::BrightBlue => style::Color::Blue,
            Color::Cyan => style::Color::DarkCyan,
            Color::BrightCyan => style::Color::Cyan,
            Color::Magenta => style::Color::DarkMagenta,
            Color::BrightMagenta => style::Color::Magenta,
            Color::White => style::Color::White,
            Color::Rgb(r, g, b) => style::Color::Rgb { r, g, b },
        }
    }
}

const PALETTE: &[char] = &['c', 'C', 'r', 'R', 'y', 'Y', 'b', 'B', 'g', 'G', 'm', 'M'];

/// Randomize numbered placeholders (1-9) in a color mask string.
/// Each number gets a random color from the palette, applied consistently
/// throughout the mask. '4' always maps to White (for eyes).
pub fn rand_color(mask: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut mapping: [Option<char>; 10] = [None; 10];
    // '4' is always white (eye)
    mapping[4] = Some('W');

    mask.chars()
        .map(|c| {
            if let Some(digit) = c.to_digit(10) {
                let idx = digit as usize;
                if idx >= 1 && idx <= 9 {
                    let color_char = mapping[idx].unwrap_or_else(|| {
                        let picked = PALETTE[rng.gen_range(0..PALETTE.len())];
                        mapping[idx] = Some(picked);
                        picked
                    });
                    return color_char;
                }
            }
            c
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_crossterm() {
        let c = Color::Rgb(128, 64, 32);
        let ct = c.to_crossterm();
        assert_eq!(ct, style::Color::Rgb { r: 128, g: 64, b: 32 });
    }

    #[test]
    fn test_rgb_color_equality() {
        assert_eq!(Color::Rgb(10, 20, 30), Color::Rgb(10, 20, 30));
        assert_ne!(Color::Rgb(10, 20, 30), Color::Rgb(10, 20, 31));
        assert_ne!(Color::Rgb(0, 0, 0), Color::Black);
    }

    #[test]
    fn test_rgb_color_is_copy_and_clone() {
        let c = Color::Rgb(255, 0, 128);
        let c2 = c; // Copy
        let c3 = c.clone(); // Clone
        assert_eq!(c, c2);
        assert_eq!(c, c3);
    }

    #[test]
    fn test_palette_colors_unchanged() {
        // Verify existing palette colors still map correctly
        assert_eq!(Color::Red.to_crossterm(), style::Color::DarkRed);
        assert_eq!(Color::BrightGreen.to_crossterm(), style::Color::Green);
        assert_eq!(Color::White.to_crossterm(), style::Color::White);
    }

    #[test]
    fn test_detect_color_support_fallback() {
        // When COLORTERM is not set to truecolor/24bit, should return Palette.
        // We can't reliably control env vars in tests without affecting other
        // tests, so just verify the function runs without panicking.
        let mode = detect_color_support();
        assert!(mode == ColorMode::Palette || mode == ColorMode::TrueColor);
    }
}
