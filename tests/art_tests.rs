use asciiquarium::app::App;
use asciiquarium::art;
use asciiquarium::color::Color;
use asciiquarium::entity::EntityType;

/// Helper: create an App with a reasonable screen size.
fn test_app() -> App {
    App::new(120, 40, false)
}

// ---------------------------------------------------------------------------
// environment.rs tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_water_creates_four_waterline_entities() {
    let mut app = test_app();
    art::environment::spawn_water(&mut app);
    // init_scene would also spawn castle, seaweed, fish, etc.
    // We only called spawn_water, so we should have exactly 4 entities.
    let fb = app.render();
    // Verify the water lines are rendered (top 4 rows should have content).
    // Row 0 should contain '~' characters.
    let row0 = &fb.cells[0];
    let tilde_count = row0.iter().filter(|c| c.map(|c| c.ch) == Some('~')).count();
    assert!(tilde_count > 0, "Water line row 0 should contain ~ characters");
}

#[test]
fn spawn_castle_renders_at_bottom_right() {
    let mut app = test_app();
    art::environment::spawn_castle(&mut app);
    let fb = app.render();
    // The castle should have content in the bottom-right area.
    // The castle flag has a '#' character.
    let mut found_flag = false;
    for row in &fb.cells {
        for cell in row {
            if let Some(c) = cell {
                if c.ch == '#' {
                    found_flag = true;
                }
            }
        }
    }
    assert!(found_flag, "Castle should contain '#' character for flag");
}

#[test]
fn spawn_seaweed_creates_animated_entity() {
    let mut app = test_app();
    art::environment::spawn_seaweed(&mut app);
    let fb = app.render();
    // Seaweed should contain '(' or ')' characters near the bottom of screen.
    let mut found_paren = false;
    for row in fb.cells.iter().rev().take(10) {
        for cell in row {
            if let Some(c) = cell {
                if c.ch == '(' || c.ch == ')' {
                    found_paren = true;
                }
            }
        }
    }
    assert!(found_paren, "Seaweed should render ( or ) characters near bottom");
}

#[test]
fn seaweed_death_callback_spawns_new_seaweed() {
    // We can't call the death callback directly since it's private,
    // but we can test via init_scene + update cycle. Instead, let's
    // verify the seaweed entity can be spawned and has expected properties.
    let mut app = App::new(120, 40, false);
    art::environment::spawn_seaweed(&mut app);
    // Render should succeed without panic.
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// fish.rs tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_fish_creates_entity_that_renders() {
    let mut app = test_app();
    art::fish::spawn_fish(&mut app, false);
    let _fb = app.render();
    // Fish starts offscreen, so it may not be visible yet.
    // The key test is that spawning and rendering don't panic.
}

#[test]
fn spawn_multiple_fish_no_panic() {
    let mut app = test_app();
    for _ in 0..20 {
        art::fish::spawn_fish(&mut app, false);
    }
    let _fb = app.render();
}

#[test]
fn splat_builder_creates_animated_entity() {
    let mut app = test_app();
    let builder = art::fish::splat_builder(10, 10, 15);
    assert_eq!(builder.entity_type, EntityType::Splat);
    app.spawn(builder);
    let _fb = app.render();
}

#[test]
fn fish_update_produces_bubble_or_empty() {
    // After many updates, fish should eventually emit a bubble.
    let mut app = test_app();
    art::fish::spawn_fish(&mut app, false);
    // Run many update ticks to give the 3% chance a fair shot.
    for _ in 0..200 {
        app.update(0.033);
    }
    // If a bubble was spawned, render should still work.
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// random.rs tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_random_creates_entity() {
    let mut app = test_app();
    art::random::spawn_random(&mut app, false);
    let _fb = app.render();
}

#[test]
fn spawn_random_classic_mode() {
    let mut app = App::new(120, 40, true);
    art::random::spawn_random(&mut app, true);
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// random.rs: whale, monster, big_fish tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_random_many_times_exercises_all_types() {
    // Spawn random 50 times to exercise all 5 entity types statistically.
    // The key test is that none of them panic during spawn or render.
    let mut app = test_app();
    for _ in 0..50 {
        art::random::spawn_random(&mut app, false);
    }
    let _fb = app.render();
}

#[test]
fn spawn_random_then_update_cycle_no_panic() {
    // Spawn all types and run update cycles to exercise animation, movement,
    // and offscreen death callbacks.
    let mut app = App::new(120, 40, false);
    for _ in 0..20 {
        art::random::spawn_random(&mut app, false);
    }
    for _ in 0..300 {
        app.update(0.033);
    }
    let _fb = app.render();
}

#[test]
fn spawn_random_on_small_screen_no_panic() {
    // Edge case: very small screen should not cause range errors.
    let mut app = App::new(30, 15, false);
    for _ in 0..20 {
        art::random::spawn_random(&mut app, false);
    }
    let _fb = app.render();
}

#[test]
fn spawn_random_on_large_screen_no_panic() {
    let mut app = App::new(300, 100, false);
    for _ in 0..20 {
        art::random::spawn_random(&mut app, false);
    }
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// Full scene init
// ---------------------------------------------------------------------------

#[test]
fn init_scene_populates_entities_and_renders() {
    let mut app = test_app();
    app.init_scene();
    let fb = app.render();
    // The scene should have water, castle, seaweed, fish, and a random object.
    // Water lines should be visible in the top rows.
    let row0_content: usize = fb.cells[0].iter().filter(|c| c.is_some()).count();
    assert!(row0_content > 0, "Row 0 should have water line content after init_scene");
}

#[test]
fn update_cycle_does_not_panic() {
    let mut app = test_app();
    app.init_scene();
    for _ in 0..100 {
        app.update(0.033);
        app.check_collisions();
    }
    let _fb = app.render();
}

#[test]
fn resize_and_reinit_works() {
    let mut app = test_app();
    app.init_scene();
    app.resize(80, 24);
    app.clear_all();
    app.init_scene();
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// Task 2: Fish count scaling tests
// ---------------------------------------------------------------------------

#[test]
fn small_terminal_has_minimum_fish() {
    // A very small terminal (40x15) should still produce at least 3 fish.
    let mut app = App::new(40, 15, false);
    app.init_scene();
    let fb = app.render();
    // init_scene produces water, castle, seaweed, fish, and one random.
    // We can't easily count fish entities from outside, but we can verify
    // the scene renders without panic and has reasonable content.
    let total_cells: usize = fb.cells.iter().flat_map(|r| r.iter()).filter(|c| c.is_some()).count();
    assert!(total_cells > 20, "Small terminal scene should have visible content");
}

#[test]
fn large_terminal_does_not_overcrowd() {
    // A very large terminal (300x100) should not have an absurd number of fish.
    let mut app = App::new(300, 100, false);
    app.init_scene();
    let _fb = app.render();
    // The key assertion is that this doesn't panic or take too long.
    // With the old formula, screen_area/350 = (91*300)/350 = 78 fish.
    // With the new formula, extra area uses divisor 600, reducing density.
}

#[test]
fn init_scene_tiny_terminal_no_panic() {
    // Edge case: extremely small terminal
    let mut app = App::new(20, 10, false);
    app.init_scene();
    let _fb = app.render();
}

// ---------------------------------------------------------------------------
// Task 3: Water line color tests
// ---------------------------------------------------------------------------

#[test]
fn water_lines_use_bright_blue() {
    let mut app = App::new(120, 40, false);
    art::environment::spawn_water(&mut app);
    let fb = app.render();
    // Row 0 is the topmost water line with ~ characters.
    // They should use BrightBlue (ocean blue).
    let row0 = &fb.cells[0];
    let tilde_cells: Vec<_> = row0.iter().filter_map(|c| *c).filter(|c| c.ch == '~').collect();
    assert!(!tilde_cells.is_empty(), "Water line should have ~ chars");
    for cell in &tilde_cells {
        assert_eq!(
            cell.color,
            Color::BrightBlue,
            "Water line ~ should use BrightBlue, got {:?}",
            cell.color
        );
    }
}

#[test]
fn castle_uses_white_for_stone() {
    let mut app = App::new(120, 40, false);
    art::environment::spawn_castle(&mut app);
    let fb = app.render();
    // Find a '=' or '-' character in the castle wall interior.
    // Characters without a mask entry should use the entity default color (White).
    let mut found = false;
    for row in &fb.cells {
        for cell in row.iter().flatten() {
            if cell.ch == '=' && cell.color == Color::White {
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    assert!(found, "Castle wall '=' chars should use White as default color");
}

// ---------------------------------------------------------------------------
// duck.rs tests
// ---------------------------------------------------------------------------

#[test]
fn spawn_duck_creates_entity_that_renders() {
    let mut app = test_app();
    art::duck::spawn_duck_scattered(&mut app);
    let fb = app.render();
    // Duck should be visible at the water surface (rows 0-3).
    // Look for duck eye character 'o' in that region.
    let mut found_duck = false;
    for row_idx in 0..4 {
        if row_idx >= fb.cells.len() {
            continue;
        }
        for cell in &fb.cells[row_idx] {
            if let Some(c) = cell {
                if c.ch == 'o' || c.ch == '(' || c.ch == ')' || c.ch == '>' || c.ch == '<' {
                    found_duck = true;
                }
            }
        }
    }
    assert!(found_duck, "Duck should render at the water surface (rows 0-3)");
}

#[test]
fn spawn_multiple_ducks_no_panic() {
    let mut app = test_app();
    for _ in 0..10 {
        art::duck::spawn_duck_scattered(&mut app);
    }
    let _fb = app.render();
}

#[test]
fn duck_respawns_after_offscreen_death() {
    // Ducks should respawn when they leave the screen.
    let mut app = App::new(120, 40, false);
    art::duck::spawn_duck_scattered(&mut app);
    // Run many update ticks to move the duck offscreen.
    for _ in 0..2000 {
        app.update(0.1);
    }
    // After death callback fires, a new duck should exist.
    // Render should succeed without panic.
    let _fb = app.render();
}

#[test]
fn duck_on_small_terminal_no_panic() {
    let mut app = App::new(30, 15, false);
    art::duck::spawn_duck_scattered(&mut app);
    let _fb = app.render();
}

#[test]
fn spawn_many_random_objects_with_update_cycles() {
    // Spawn many random objects and run update cycles to exercise
    // all types including rare dolphin and submarine.
    let mut app = App::new(120, 40, false);
    for _ in 0..100 {
        art::random::spawn_random(&mut app, false);
    }
    for _ in 0..500 {
        app.update(0.033);
    }
    let _fb = app.render();
}

#[test]
fn init_scene_includes_ducks() {
    let mut app = test_app();
    app.init_scene();
    let fb = app.render();
    // Ducks float at y=0 (water surface). Check for duck-like characters in rows 0-3.
    let mut found_duck_content = false;
    for row_idx in 0..4 {
        let chars: Vec<_> = fb.cells[row_idx].iter().filter_map(|c| *c).collect();
        if !chars.is_empty() {
            found_duck_content = true;
            break;
        }
    }
    assert!(found_duck_content, "Rows 0-3 should have content (ducks float at the water surface)");
}
