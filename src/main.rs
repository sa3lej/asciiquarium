use std::path::PathBuf;
use std::time::Instant;

use crossterm::terminal;

mod app;
mod art;
mod color;
mod entity;
mod image_to_ascii;
mod input;
mod renderer;
mod shape;

use app::App;
use input::Action;
use renderer::Renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let classic = args.iter().any(|a| a == "-c" || a == "--classic");

    // Collect --image paths and --image-size
    let mut custom_images: Vec<PathBuf> = Vec::new();
    let mut image_size: Option<(u16, u16)> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--image" | "-i" => {
                if i + 1 < args.len() {
                    custom_images.push(PathBuf::from(&args[i + 1]));
                    i += 2;
                    continue;
                }
            }
            "--image-size" => {
                if i + 1 < args.len() {
                    if let Some((w, h)) = parse_size(&args[i + 1]) {
                        image_size = Some((w, h));
                    } else {
                        eprintln!("Invalid --image-size format. Use WxH (e.g., 60x30)");
                        return Ok(());
                    }
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let mut renderer = Renderer::new();
    renderer.init()?;

    let (width, height) = terminal::size()?;
    let mut app = App::new(width, height, classic);

    // Pre-load custom images
    let color_mode = color::detect_color_support();
    let convert_config = image_to_ascii::ConvertConfig {
        color_mode,
        max_width: image_size.map(|(w, _)| w).or(Some(40)),
        max_height: image_size.map(|(_, h)| h).or(Some(20)),
        ..image_to_ascii::ConvertConfig::default()
    };
    let mut custom_shapes = Vec::new();
    for path in &custom_images {
        match image_to_ascii::load_image_as_shape(path, &convert_config) {
            Ok(shape) => custom_shapes.push(shape),
            Err(e) => {
                renderer.cleanup()?;
                eprintln!("Error loading image {}: {}", path.display(), e);
                return Err(e.into());
            }
        }
    }

    // Register custom images
    for shape in custom_shapes {
        app.add_custom_shape(shape);
    }

    let mut speed: f64 = 1.0;

    'outer: loop {
        app.init_scene();

        renderer.clear()?;

        let mut last_tick = Instant::now();

        loop {
            let timeout = std::time::Duration::from_millis(100);
            match input::poll_input(timeout) {
                Action::Quit => break 'outer,
                Action::Redraw => {
                    let (w, h) = terminal::size()?;
                    app.resize(w, h);
                    break; // restart scene
                }
                Action::TogglePause => app.paused = !app.paused,
                Action::SpeedUp => {
                    speed = (speed * 1.5).min(20.0);
                }
                Action::SlowDown => {
                    speed = (speed / 1.5).max(0.1);
                }
                Action::None => {}
            }

            if !app.paused {
                let now = Instant::now();
                let dt = now.duration_since(last_tick).as_secs_f64() * speed;
                last_tick = now;
                app.update(dt);
                app.check_collisions();
            }

            let frame = app.render();
            renderer.draw(&frame)?;
        }

        app.clear_all();
    }

    renderer.cleanup()?;
    Ok(())
}

fn parse_size(s: &str) -> Option<(u16, u16)> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse().ok()?;
        let h = parts[1].parse().ok()?;
        Some((w, h))
    } else {
        None
    }
}
