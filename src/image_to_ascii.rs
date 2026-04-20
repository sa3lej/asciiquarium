use std::process::Command;

use image::{DynamicImage, GenericImageView, Rgba};

use crate::color::{Color, ColorMode};
use crate::shape::{Frame, Shape};

/// Characters ordered by visual density (light to heavy).
/// Space is not included — transparent pixels become None in the Frame.
const DENSITY_CHARS: &[char] = &[
    '.', '\'', '`', ',', ':', ';', '!', '|', '/', '\\', '~', '-', '_', '+', '*', '<', '>',
    '=', '?', '^', '"', '(', ')', '{', '}', '[', ']', '#', '%', '&', '@',
];

/// Configuration for image-to-ASCII conversion.
pub struct ConvertConfig {
    /// Maximum width in terminal columns. If None, uses image width.
    pub max_width: Option<u16>,
    /// Maximum height in terminal rows. If None, uses image height / 2.
    pub max_height: Option<u16>,
    /// Character to use for transparent/alpha pixels. None = transparent in entity.
    pub transparent_threshold: u8,
    /// Whether to use color (if false, everything uses default_color).
    pub color: bool,
    /// Color depth for the conversion output.
    pub color_mode: ColorMode,
}

impl Default for ConvertConfig {
    fn default() -> Self {
        ConvertConfig {
            max_width: Some(40),
            max_height: Some(20),
            transparent_threshold: 128,
            color: true,
            color_mode: ColorMode::TrueColor,
        }
    }
}

/// Convert an image file to a Shape suitable for use as an entity.
pub fn image_to_shape(img: &DynamicImage, config: &ConvertConfig) -> Shape {
    let (img_w, img_h) = img.dimensions();

    // Terminal cells are roughly 2:1 (height:width), so we halve the vertical resolution
    let aspect_correction = 2.0;

    let target_w = config.max_width.unwrap_or(img_w as u16) as f64;
    let target_h = config.max_height.unwrap_or((img_h as f64 / aspect_correction) as u16) as f64;

    // Scale to fit within bounds while preserving aspect ratio
    let scale_x = target_w / img_w as f64;
    let scale_y = target_h / (img_h as f64 / aspect_correction);
    let scale = scale_x.min(scale_y);

    let out_w = (img_w as f64 * scale).round() as u16;
    let out_h = (img_h as f64 / aspect_correction * scale).round() as u16;

    let out_w = out_w.max(1);
    let out_h = out_h.max(1);

    let mut cells: Vec<Vec<Option<char>>> = Vec::with_capacity(out_h as usize);
    let mut colors: Vec<Vec<Option<Color>>> = Vec::with_capacity(out_h as usize);

    for row in 0..out_h {
        let mut cell_row = Vec::with_capacity(out_w as usize);
        let mut color_row = Vec::with_capacity(out_w as usize);

        for col in 0..out_w {
            // Map output cell to source image region
            let src_x = (col as f64 / scale).round() as u32;
            let src_y = (row as f64 * aspect_correction / scale / aspect_correction)
                .round() as u32;

            // Sample a block of pixels and average them
            let block_w = (1.0 / scale).ceil() as u32;
            let block_h = (aspect_correction / scale).ceil() as u32;

            let (avg_r, avg_g, avg_b, avg_a) =
                sample_block(img, src_x, src_y, block_w, block_h);

            if avg_a < config.transparent_threshold {
                cell_row.push(None);
                color_row.push(None);
            } else {
                // Map brightness to character
                let brightness =
                    (0.299 * avg_r as f64 + 0.587 * avg_g as f64 + 0.114 * avg_b as f64) / 255.0;
                let char_idx =
                    (brightness * (DENSITY_CHARS.len() - 1) as f64).round() as usize;
                let ch = DENSITY_CHARS[char_idx.min(DENSITY_CHARS.len() - 1)];

                cell_row.push(Some(ch));

                if config.color {
                    let c = match config.color_mode {
                        ColorMode::TrueColor => Color::Rgb(avg_r, avg_g, avg_b),
                        ColorMode::Palette => rgb_to_terminal_color(avg_r, avg_g, avg_b),
                    };
                    color_row.push(Some(c));
                } else {
                    color_row.push(None);
                }
            }
        }

        cells.push(cell_row);
        colors.push(color_row);
    }

    Shape {
        frames: vec![Frame {
            cells,
            colors,
            width: out_w,
            height: out_h,
        }],
    }
}

/// Try to load an image using chafa for high-quality Unicode terminal art.
///
/// Shells out to `chafa <path> --size WxH --format symbols --color-space rgb --fg-only`
/// and parses the ANSI output into a Shape. Chafa produces Unicode block characters
/// (like ▄▀█) with truecolor escape sequences, yielding much better visual quality
/// than our built-in brightness-to-character mapper.
///
/// Returns `None` if chafa is not installed or fails to run.
pub fn load_image_with_chafa(
    path: &std::path::Path,
    config: &ConvertConfig,
) -> Option<Shape> {
    let width = config.max_width.unwrap_or(40);
    let height = config.max_height.unwrap_or(20);
    let size_arg = format!("{}x{}", width, height);

    let output = Command::new("chafa")
        .arg(path.as_os_str())
        .args(["--size", &size_arg])
        .args(["--format", "symbols"])
        .args(["--color-space", "rgb"])
        .arg("--fg-only")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    Some(parse_chafa_output(&text))
}

/// Parse chafa's ANSI output into a Shape.
///
/// Chafa emits truecolor foreground sequences of the form `\x1b[38;2;R;G;Bm`
/// followed by characters. Space characters are treated as transparent (None).
/// A reset sequence `\x1b[0m` clears the current color.
fn parse_chafa_output(ansi_text: &str) -> Shape {
    let mut cells: Vec<Vec<Option<char>>> = Vec::new();
    let mut colors: Vec<Vec<Option<Color>>> = Vec::new();
    let mut max_width: usize = 0;

    for line in ansi_text.lines() {
        let (cell_row, color_row) = parse_chafa_line(line);
        if cell_row.len() > max_width {
            max_width = cell_row.len();
        }
        cells.push(cell_row);
        colors.push(color_row);
    }

    // Pad rows to uniform width
    for row in &mut cells {
        row.resize(max_width, None);
    }
    for row in &mut colors {
        row.resize(max_width, None);
    }

    let height = cells.len() as u16;
    let width = max_width as u16;

    Shape {
        frames: vec![Frame {
            cells,
            colors,
            width,
            height,
        }],
    }
}

/// Parse a single line of chafa ANSI output into cells and colors.
///
/// Handles escape sequences:
/// - `\x1b[38;2;R;G;Bm` — set foreground to RGB
/// - `\x1b[0m` — reset color
/// - `\x1b[...m` — other SGR sequences (ignored but consumed)
fn parse_chafa_line(line: &str) -> (Vec<Option<char>>, Vec<Option<Color>>) {
    let mut cell_row: Vec<Option<char>> = Vec::new();
    let mut color_row: Vec<Option<Color>> = Vec::new();
    let mut current_color: Option<Color> = None;

    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '\x1b' && i + 1 < len && chars[i + 1] == '[' {
            // Parse CSI escape sequence: consume until a letter (the final byte)
            let start = i + 2; // skip ESC [
            let mut end = start;
            while end < len && !chars[end].is_ascii_alphabetic() {
                end += 1;
            }
            if end < len {
                let final_byte = chars[end];
                let params: String = chars[start..end].iter().collect();

                if final_byte == 'm' {
                    // SGR (Select Graphic Rendition)
                    if params == "0" || params.is_empty() {
                        current_color = None;
                    } else if params.starts_with("38;2;") {
                        // Truecolor: 38;2;R;G;B
                        let parts: Vec<&str> = params.splitn(5, ';').collect();
                        if parts.len() == 5 {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                parts[2].parse::<u8>(),
                                parts[3].parse::<u8>(),
                                parts[4].parse::<u8>(),
                            ) {
                                current_color = Some(Color::Rgb(r, g, b));
                            }
                        }
                    }
                    // else: other SGR codes (bold, reverse, etc.) — ignore
                }
                // All other CSI sequences (?25h, ?25l, etc.) — silently consume
                i = end + 1; // skip past final byte
            } else {
                // Malformed escape — skip ESC char
                i += 1;
            }
        } else {
            let ch = chars[i];
            if ch == ' ' {
                cell_row.push(None);
                color_row.push(None);
            } else {
                cell_row.push(Some(ch));
                color_row.push(current_color);
            }
            i += 1;
        }
    }

    (cell_row, color_row)
}

/// Load an image from a file path and convert it to a Shape.
///
/// Tries chafa first for high-quality Unicode output. Falls back to the
/// built-in brightness-to-character converter if chafa is not available.
pub fn load_image_as_shape(
    path: &std::path::Path,
    config: &ConvertConfig,
) -> Result<Shape, image::ImageError> {
    // Try chafa first — produces much better quality with Unicode block chars
    if let Some(shape) = load_image_with_chafa(path, config) {
        return Ok(shape);
    }

    // Fallback to built-in converter
    let img = image::open(path)?;
    Ok(image_to_shape(&img, config))
}

/// Sample a block of pixels and return the average RGBA values.
fn sample_block(img: &DynamicImage, x: u32, y: u32, w: u32, h: u32) -> (u8, u8, u8, u8) {
    let (img_w, img_h) = img.dimensions();
    let mut total_r: u64 = 0;
    let mut total_g: u64 = 0;
    let mut total_b: u64 = 0;
    let mut total_a: u64 = 0;
    let mut count: u64 = 0;

    for dy in 0..h {
        for dx in 0..w {
            let px = (x + dx).min(img_w - 1);
            let py = (y + dy).min(img_h - 1);
            let Rgba([r, g, b, a]) = img.get_pixel(px, py);
            total_r += r as u64;
            total_g += g as u64;
            total_b += b as u64;
            total_a += a as u64;
            count += 1;
        }
    }

    if count == 0 {
        return (0, 0, 0, 0);
    }

    (
        (total_r / count) as u8,
        (total_g / count) as u8,
        (total_b / count) as u8,
        (total_a / count) as u8,
    )
}

/// Map an RGB color to the closest terminal color from our 14-color palette.
fn rgb_to_terminal_color(r: u8, g: u8, b: u8) -> Color {
    // Terminal color palette with approximate RGB values
    let palette: &[(Color, (u8, u8, u8))] = &[
        (Color::Black, (0, 0, 0)),
        (Color::Red, (170, 0, 0)),
        (Color::BrightRed, (255, 85, 85)),
        (Color::Green, (0, 170, 0)),
        (Color::BrightGreen, (85, 255, 85)),
        (Color::Yellow, (170, 170, 0)),
        (Color::BrightYellow, (255, 255, 85)),
        (Color::Blue, (0, 0, 170)),
        (Color::BrightBlue, (85, 85, 255)),
        (Color::Cyan, (0, 170, 170)),
        (Color::BrightCyan, (85, 255, 255)),
        (Color::Magenta, (170, 0, 170)),
        (Color::BrightMagenta, (255, 85, 255)),
        (Color::White, (255, 255, 255)),
    ];

    let mut best_color = Color::White;
    let mut best_dist = u64::MAX;

    for &(color, (pr, pg, pb)) in palette {
        let dr = (r as i32 - pr as i32).pow(2) as u64;
        let dg = (g as i32 - pg as i32).pow(2) as u64;
        let db = (b as i32 - pb as i32).pow(2) as u64;
        // Weight green channel more (human eye sensitivity)
        let dist = dr + 2 * dg + db;
        if dist < best_dist {
            best_dist = dist;
            best_color = color;
        }
    }

    best_color
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorMode;

    #[test]
    fn test_rgb_to_terminal_color() {
        // White and black should be exact
        assert_eq!(rgb_to_terminal_color(255, 255, 255), Color::White);
        assert_eq!(rgb_to_terminal_color(0, 0, 0), Color::Black);
        // Reddish colors should map to red variants
        let red = rgb_to_terminal_color(255, 0, 0);
        assert!(red == Color::Red || red == Color::BrightRed);
        // Greenish colors should map to green variants
        let green = rgb_to_terminal_color(0, 255, 0);
        assert!(green == Color::Green || green == Color::BrightGreen);
        // Blueish colors should map to blue variants
        let blue = rgb_to_terminal_color(0, 0, 255);
        assert!(blue == Color::Blue || blue == Color::BrightBlue);
        // Yellow should be recognizable
        let yellow = rgb_to_terminal_color(255, 255, 0);
        assert!(yellow == Color::Yellow || yellow == Color::BrightYellow);
    }

    #[test]
    fn test_convert_creates_valid_shape() {
        // Create a small test image (4x4 red square)
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            Rgba([255, 0, 0, 255])
        }));

        let config = ConvertConfig {
            max_width: Some(4),
            max_height: Some(2),
            color: true,
            ..Default::default()
        };

        let shape = image_to_shape(&img, &config);
        assert_eq!(shape.frames.len(), 1);
        let frame = &shape.frames[0];
        assert!(frame.width > 0);
        assert!(frame.height > 0);

        // All cells should be non-transparent (solid red image)
        for row in &frame.cells {
            for cell in row {
                assert!(cell.is_some(), "Solid image should have no transparent cells");
            }
        }
    }

    #[test]
    fn test_transparent_pixels() {
        // Create image with transparent pixels
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            Rgba([255, 0, 0, 0]) // fully transparent
        }));

        let config = ConvertConfig {
            max_width: Some(4),
            max_height: Some(2),
            color: true,
            ..Default::default()
        };

        let shape = image_to_shape(&img, &config);
        let frame = &shape.frames[0];

        // All cells should be transparent
        for row in &frame.cells {
            for cell in row {
                assert!(cell.is_none(), "Transparent image should have no visible cells");
            }
        }
    }

    #[test]
    fn test_truecolor_mode_produces_rgb_colors() {
        // Create a solid-color image with a specific RGB value
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            Rgba([100, 150, 200, 255])
        }));

        let config = ConvertConfig {
            max_width: Some(4),
            max_height: Some(2),
            color: true,
            color_mode: ColorMode::TrueColor,
            ..Default::default()
        };

        let shape = image_to_shape(&img, &config);
        let frame = &shape.frames[0];

        // Every visible cell should have an Rgb color
        for row in &frame.colors {
            for color in row {
                if let Some(c) = color {
                    match c {
                        Color::Rgb(_, _, _) => {} // expected
                        other => panic!("Expected Rgb color in TrueColor mode, got {:?}", other),
                    }
                }
            }
        }
    }

    #[test]
    fn test_palette_mode_produces_palette_colors() {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_fn(4, 4, |_, _| {
            Rgba([100, 150, 200, 255])
        }));

        let config = ConvertConfig {
            max_width: Some(4),
            max_height: Some(2),
            color: true,
            color_mode: ColorMode::Palette,
            ..Default::default()
        };

        let shape = image_to_shape(&img, &config);
        let frame = &shape.frames[0];

        // Every visible cell should NOT have an Rgb color
        for row in &frame.colors {
            for color in row {
                if let Some(c) = color {
                    match c {
                        Color::Rgb(_, _, _) => {
                            panic!("Palette mode should not produce Rgb colors")
                        }
                        _ => {} // expected: one of the 14 palette colors
                    }
                }
            }
        }
    }

    #[test]
    fn test_default_config_uses_truecolor() {
        let config = ConvertConfig::default();
        assert_eq!(config.color_mode, ColorMode::TrueColor);
    }

    #[test]
    fn test_parse_chafa_line_plain_text() {
        let (cells, colors) = super::parse_chafa_line("ABC");
        assert_eq!(cells, vec![Some('A'), Some('B'), Some('C')]);
        // No ANSI escapes, so all colors should be None
        assert_eq!(colors, vec![None, None, None]);
    }

    #[test]
    fn test_parse_chafa_line_spaces_are_transparent() {
        let (cells, colors) = super::parse_chafa_line("A B");
        assert_eq!(cells, vec![Some('A'), None, Some('B')]);
        assert_eq!(colors, vec![None, None, None]);
    }

    #[test]
    fn test_parse_chafa_line_truecolor_escape() {
        // ESC[38;2;255;128;0m followed by a character
        let line = "\x1b[38;2;255;128;0mX";
        let (cells, colors) = super::parse_chafa_line(line);
        assert_eq!(cells, vec![Some('X')]);
        assert_eq!(colors, vec![Some(Color::Rgb(255, 128, 0))]);
    }

    #[test]
    fn test_parse_chafa_line_reset_clears_color() {
        let line = "\x1b[38;2;10;20;30mA\x1b[0mB";
        let (cells, colors) = super::parse_chafa_line(line);
        assert_eq!(cells, vec![Some('A'), Some('B')]);
        assert_eq!(colors, vec![Some(Color::Rgb(10, 20, 30)), None]);
    }

    #[test]
    fn test_parse_chafa_line_multiple_colors() {
        let line = "\x1b[38;2;255;0;0mR\x1b[38;2;0;255;0mG\x1b[38;2;0;0;255mB";
        let (cells, colors) = super::parse_chafa_line(line);
        assert_eq!(cells, vec![Some('R'), Some('G'), Some('B')]);
        assert_eq!(
            colors,
            vec![
                Some(Color::Rgb(255, 0, 0)),
                Some(Color::Rgb(0, 255, 0)),
                Some(Color::Rgb(0, 0, 255)),
            ]
        );
    }

    #[test]
    fn test_parse_chafa_output_multiline() {
        let ansi = "\x1b[38;2;255;0;0mAB\n\x1b[38;2;0;255;0mCD";
        let shape = super::parse_chafa_output(ansi);
        assert_eq!(shape.frames.len(), 1);
        let frame = &shape.frames[0];
        assert_eq!(frame.height, 2);
        assert_eq!(frame.width, 2);
        assert_eq!(frame.cells[0], vec![Some('A'), Some('B')]);
        assert_eq!(frame.cells[1], vec![Some('C'), Some('D')]);
        assert_eq!(
            frame.colors[0],
            vec![Some(Color::Rgb(255, 0, 0)), Some(Color::Rgb(255, 0, 0))]
        );
        assert_eq!(
            frame.colors[1],
            vec![Some(Color::Rgb(0, 255, 0)), Some(Color::Rgb(0, 255, 0))]
        );
    }

    #[test]
    fn test_parse_chafa_output_pads_uneven_rows() {
        let ansi = "ABC\nD";
        let shape = super::parse_chafa_output(ansi);
        let frame = &shape.frames[0];
        assert_eq!(frame.width, 3);
        assert_eq!(frame.cells[1].len(), 3);
        // Padded cells should be None (transparent)
        assert_eq!(frame.cells[1][1], None);
        assert_eq!(frame.cells[1][2], None);
    }

    #[test]
    fn test_parse_chafa_line_unicode_block_chars() {
        // Chafa commonly outputs Unicode block characters
        let line = "\x1b[38;2;100;200;50m\u{2584}\u{2580}\u{2588}";
        let (cells, colors) = super::parse_chafa_line(line);
        assert_eq!(cells, vec![Some('\u{2584}'), Some('\u{2580}'), Some('\u{2588}')]);
        let expected_color = Some(Color::Rgb(100, 200, 50));
        assert_eq!(colors, vec![expected_color, expected_color, expected_color]);
    }

    #[test]
    fn test_parse_chafa_output_empty_input() {
        let shape = super::parse_chafa_output("");
        let frame = &shape.frames[0];
        // Empty string produces zero lines
        assert_eq!(frame.height, 0);
        assert_eq!(frame.width, 0);
    }

    #[test]
    fn test_load_image_with_chafa_nonexistent_file() {
        let config = ConvertConfig::default();
        let result = super::load_image_with_chafa(
            std::path::Path::new("/nonexistent/image.png"),
            &config,
        );
        // Should return None (chafa fails on nonexistent file)
        assert!(result.is_none());
    }
}
