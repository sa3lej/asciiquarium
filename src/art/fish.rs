use rand::Rng;

use crate::app::App;
use crate::color::{self, Color};
use crate::entity::{
    DeathContext, Entity, EntityBuilder, EntityCommand, EntityType, UpdateContext,
};
use crate::shape::Shape;

// ---------------------------------------------------------------------------
// Fish art definitions: (right_shape, right_mask, left_shape, left_mask)
// ---------------------------------------------------------------------------

struct FishDef {
    right_art: &'static str,
    right_mask: &'static str,
    left_art: &'static str,
    left_mask: &'static str,
    /// Character treated as transparent in the art (e.g. '?' for new fish).
    auto_trans: Option<char>,
}

/// Helper to build a FishDef with no transparency character.
const fn fish(
    right_art: &'static str,
    right_mask: &'static str,
    left_art: &'static str,
    left_mask: &'static str,
) -> FishDef {
    FishDef {
        right_art,
        right_mask,
        left_art,
        left_mask,
        auto_trans: Some(' '),
    }
}

/// Helper to build a FishDef with a transparency character.
const fn fish_trans(
    right_art: &'static str,
    right_mask: &'static str,
    left_art: &'static str,
    left_mask: &'static str,
    trans: char,
) -> FishDef {
    FishDef {
        right_art,
        right_mask,
        left_art,
        left_mask,
        auto_trans: Some(trans),
    }
}

// ---------------------------------------------------------------------------
// Old fish (from original asciiquarium Perl source, add_old_fish)
// ---------------------------------------------------------------------------

const OLD_FISH_DEFS: &[FishDef] = &[
    // ---- Old Fish 0: big fish with dots ----
    fish(
        "       \\\n     ...\\..,\n\\  /'       \\\n >=     (  ' >\n/  \\      / /\n    `\"'\"'/''",
        "       2\n     1112111\n6  11       1\n 66     7  4 5\n6  1      3 1\n    11111311",
        "      /\n  ,../...\n /       '\\  /\n< '  )     =<\n \\ \\      /  \\\n  `'\\'\"'\"'",
        "      2\n  1112111\n 1       11  6\n5 4  7     66\n 1 3      1  6\n  11311111",
    ),
    // ---- Old Fish 1: small with eye ----
    fish(
        "    \\\n\\ /--\\\n>=  (o>\n/ \\__/\n    /",
        "    2\n6 1111\n66  745\n6 1111\n    3",
        "  /\n /--\\ /\n<o)  =<\n \\__/ \\\n  \\",
        "  2\n 1111 6\n547  66\n 1111 6\n  3",
    ),
    // ---- Old Fish 2: spiny jellyfish ----
    fish(
        "       \\:.\n\\;,   ,;\\\\\\,,\n  \\\\\\;;:::::::o\n  ///;;::::::::<\n /;` ``/////``",
        "       222\n666   1122211\n  6661111111114\n  66611111111115\n 666 113333311",
        "      .:/\n   ,,///;,   ,;/\n o:::::::;;///\n>::::::::;;\\\\\\\n  ''\\\\\\\\\\'' ';\\",
        "      222\n   1122211   666\n 4111111111666\n51111111111666\n  113333311 666",
    ),
    // ---- Old Fish 3: tiny fish ----
    fish(
        "  __\n><_'>\n   '",
        "  11\n61145\n   3",
        " __\n<'_><\n `",
        " 11\n54116\n 3",
    ),
    // ---- Old Fish 4: small with dots ----
    fish(
        "   ..\\,\n>='   ('>\n  '''/''",
        "   1121\n661   745\n  111311",
        "  ,/..\n<')   `=<\n ``\\```",
        "  1211\n547   166\n 113111",
    ),
    // ---- Old Fish 5: dorsal fin fish ----
    fish(
        "   \\\n  / \\\n>=_('>\n  \\_/\n   /",
        "   2\n  1 1\n661745\n  111\n   3",
        "  /\n / \\\n<')_=<\n \\_/\n  \\",
        "  2\n 1 1\n547166\n 111\n  3",
    ),
    // ---- Old Fish 6: tiny comma fish ----
    fish(
        "  ,\\\n>=('>\n  '/",
        "  12\n66745\n  13",
        " /,\n<')=<\n \\`",
        " 21\n54766\n 31",
    ),
    // ---- Old Fish 7: flat fish ----
    fish(
        "  __\n\\/ o\\\n/\\__/",
        "  11\n61 41\n61111",
        " __\n/o \\/\n\\__/\\",
        " 11\n14 16\n11116",
    ),
];

// ---------------------------------------------------------------------------
// New fish (from Android wallpaper port in Perl source, add_new_fish)
// ---------------------------------------------------------------------------

const NEW_FISH_DEFS: &[FishDef] = &[
    // ---- New Fish 0: dorsal fin variant ----
    fish(
        "   \\\n  / \\\n>=_('>\n  \\_/\n   /",
        "   1\n  1 1\n663745\n  111\n   3",
        "  /\n / \\\n<')_=<\n \\_/\n  \\",
        "  2\n 111\n547366\n 111\n  3",
    ),
    // ---- New Fish 1: curly braces fish ----
    fish(
        "     ,\n     }\\\n\\  .'  `\\\n}}<   ( 6>\n/  `,  .'\n     }/\n     '",
        "     2\n     22\n6  11  11\n661   7 45\n6  11  11\n     33\n     3",
        "    ,\n   /{\n /'  `.  /\n<6 )   >{{\n `.  ,'  \\\n   \\{\n    `",
        "    2\n   22\n 11  11  6\n54 7   166\n 11  11  6\n   33\n    3",
    ),
    // ---- New Fish 2: big pufferfish (uses '?' as transparency) ----
    fish_trans(
        "            \\'`.\n             )  \\\n(`.??????_.-`' ' '`-.\n \\ `.??.`        (o) \\_\n  >  ><     (((       (\n / .`??`._      /_|  /'\n(`.???????`-. _  _.-`\n            /__/'\n",
        "            1111\n             1  1\n111      11111 1 1111\n 1 11  11        141 11\n  1  11     777       5\n 1 11  111      333  11\n111       111 1  1111\n            11111\n",
        "       .'`/\n      /  (\n  .-'` ` `'-._??????.')\n_/ (o)        '.??.' /\n)       )))     ><  <\n`\\  |_\\      _.'??'. \\\n  '-._  _ .-'???????'.)\n      `\\__\\",
        "       1111\n      1  1\n  1111 1 11111      111\n11 141        11  11 1\n5       777     11  1\n11  333      111  11 1\n  1111  1 111       111\n      11111",
        '?',
    ),
    // ---- New Fish 3: fish with nose ----
    fish(
        "       ,--,_\n__    _\\.---'-.\n\\ '.-\"     // o\\\n/_.'-._    \\\\  /\n       `\"--(/\"`",
        "       22222\n66    121111211\n6 6111     77 41\n6661111    77  1\n       11113311",
        "    _,--,\n .-'---./_    __\n/o \\\\     \"-.' /\n\\  //    _.-'._\\\n `\"\\)--\"`",
        "    22222\n 112111121    66\n14 77     1116 6\n1  77    1111666\n 11331111",
    ),
];

// ---------------------------------------------------------------------------
// Extra fish (invented for the Rust port, not in original Perl)
// ---------------------------------------------------------------------------

const EXTRA_FISH_DEFS: &[FishDef] = &[
    // ---- Extra 0: small rounded ----
    fish(
        "  _____\n><_____'>\n  `---'",
        "  11111\n611111145\n  13331",
        " _____\n<'_____><\n  `---'",
        " 11111\n541111116\n  13331",
    ),
    // ---- Extra 1: puffer ----
    fish(
        "  ___\n><((('>\n  ---",
        "  111\n6111145\n  333",
        " ___\n<')))><\n ---",
        " 111\n5411116\n 333",
    ),
    // ---- Extra 2: long fish ----
    fish(
        "    \\  \\\n  ___\\  \\\n><____'  '>\n  ---/  /\n    /  /",
        "    2  2\n  1112  1\n6111111  45\n  3331  3\n    3  3",
        "  /  /\n /  /___\n<'  '____><\n  \\  \\---\n    \\  \\",
        "  3  3\n 3  1111\n54  1111116\n  3  3331\n    2  2",
    ),
    // ---- Extra 3: angel fish ----
    fish(
        "   /\\\n  /--\\\n><----'>\n  \\--/\n   \\/",
        "   11\n  1111\n61111145\n  1111\n   11",
        "  /\\\n /--\\\n<'----><\n /--\\\n  \\/",
        "  11\n 1111\n54111116\n 1111\n  11",
    ),
    // ---- Extra 4: tiny simple ----
    fish(
        "><>",
        "614",
        "<><",
        "416",
    ),
    // ---- Extra 5: round little ----
    fish(
        "  __\n>|  '>\n  --",
        "  11\n611145\n  33",
        " __\n<'  |<\n --",
        " 11\n541116\n 33",
    ),
];

// ---------------------------------------------------------------------------
// Bubble
// ---------------------------------------------------------------------------

fn bubble_builder(x: f64, y: f64, z: i32) -> EntityBuilder {
    // Bubbles animate: . → o → O → O → O (grow as they rise)
    let frames: Vec<(&str, Option<&str>)> = vec![
        (".", None),
        ("o", None),
        ("O", None),
        ("O", None),
        ("O", None),
    ];
    let shape = Shape::multi(frames, None);
    EntityBuilder::new(EntityType::Bubble, "bubble", shape)
        .position(x, y, z - 1) // bubble always on top of the fish
        .velocity(0.0, -1.0)
        .color(Color::BrightCyan)
        .die_offscreen()
        .with_physics()
        .animate(0.5)
}

// ---------------------------------------------------------------------------
// Splat (multi-frame death animation)
// ---------------------------------------------------------------------------

const SPLAT_FRAMES: [&str; 4] = [
    " *",
    "***\n *",
    "  *\n*****\n  *",
    "   *\n *****\n*******\n *****\n   *",
];

pub fn splat_builder(x: i32, y: i32, z: i32) -> EntityBuilder {
    let frames: Vec<(&str, Option<&str>)> = SPLAT_FRAMES.iter().map(|f| (*f, None)).collect();
    let shape = Shape::multi(frames, None);
    EntityBuilder::new(EntityType::Splat, "splat", shape)
        .position(x as f64, y as f64, z)
        .color(Color::Red)
        .animate(0.25)
        .on_update(splat_update)
}

/// Splat dies after cycling through all frames once.
fn splat_update(entity: &mut Entity, _ctx: &UpdateContext) -> Vec<EntityCommand> {
    // The animation system advances frames automatically.
    // Die once we have completed a full cycle (reached last frame and timer is about to wrap).
    let total_frames = entity.shape.frames.len();
    if entity.current_frame == total_frames - 1
        && entity.frame_timer + _ctx.dt >= entity.animation_speed
    {
        return vec![EntityCommand::KillSelf];
    }
    vec![]
}

// ---------------------------------------------------------------------------
// Fish update callback (bubble emission)
// ---------------------------------------------------------------------------

fn fish_update(entity: &mut Entity, ctx: &UpdateContext) -> Vec<EntityCommand> {
    let mut rng = rand::thread_rng();
    // ~3% chance per tick (matching Perl's rand(100) > 97)
    if rng.gen::<f64>() < 0.03 * ctx.dt * 10.0 {
        let frame = &entity.shape.frames[entity.current_frame];
        // Bubble spawns at the mouth (front) of the fish, vertically centered
        let bx = if entity.vx > 0.0 {
            entity.x + frame.width as f64
        } else {
            entity.x - 1.0
        };
        let by = entity.y + (frame.height as f64 / 2.0);
        return vec![EntityCommand::Spawn(bubble_builder(bx, by, entity.z))];
    }
    vec![]
}

// ---------------------------------------------------------------------------
// Fish death callback (respawn a new fish)
// ---------------------------------------------------------------------------

fn fish_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_fish(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

// ---------------------------------------------------------------------------
// Fish spawning
// ---------------------------------------------------------------------------

fn build_fish_at(screen_w: u16, screen_h: u16, classic: bool, scatter: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();

    // Select a fish definition. In classic mode, only use old fish.
    // In non-classic mode, mix old + new + extra: ~75% old/extra, ~25% new
    // (matches Perl: rand(12) > 8 gives ~25% new).
    let def = if classic {
        // Classic mode: old fish + extra fish only
        let total = OLD_FISH_DEFS.len() + EXTRA_FISH_DEFS.len();
        let idx = rng.gen_range(0..total);
        if idx < OLD_FISH_DEFS.len() {
            &OLD_FISH_DEFS[idx]
        } else {
            &EXTRA_FISH_DEFS[idx - OLD_FISH_DEFS.len()]
        }
    } else if rng.gen_range(0..12) > 8 {
        // ~25% chance: pick a new fish
        let idx = rng.gen_range(0..NEW_FISH_DEFS.len());
        &NEW_FISH_DEFS[idx]
    } else {
        // ~75% chance: pick an old or extra fish
        let total = OLD_FISH_DEFS.len() + EXTRA_FISH_DEFS.len();
        let idx = rng.gen_range(0..total);
        if idx < OLD_FISH_DEFS.len() {
            &OLD_FISH_DEFS[idx]
        } else {
            &EXTRA_FISH_DEFS[idx - OLD_FISH_DEFS.len()]
        }
    };

    let going_right = rng.gen_bool(0.5);

    let (art, mask_str) = if going_right {
        (def.right_art, def.right_mask)
    } else {
        (def.left_art, def.left_mask)
    };

    // Randomize colors via numbered mask placeholders
    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), def.auto_trans);

    let frame = &shape.frames[0];
    let speed = rng.gen_range(0.5..2.0_f64);
    let vx = if going_right { speed } else { -speed };

    // For initial scene setup (scatter=true), spread fish across the whole screen.
    // For respawns (scatter=false), start just offscreen on the appropriate side.
    let x = if scatter {
        rng.gen_range(0.0..(screen_w as f64))
    } else if going_right {
        -(frame.width as f64)
    } else {
        screen_w as f64
    };

    // Water occupies rows 0-3, so fish swim from row 9+ (leaving a clear gap).
    let min_y = 9.0;
    let max_y = (screen_h as f64) - (frame.height as f64) - 2.0;
    let y = if max_y > min_y {
        // Bias toward middle: average two random values for bell-curve distribution
        let r1 = rng.gen_range(0.0..1.0_f64);
        let r2 = rng.gen_range(0.0..1.0_f64);
        min_y + ((r1 + r2) / 2.0) * (max_y - min_y)
    } else {
        min_y
    };

    let z = rng.gen_range(10..=20);

    EntityBuilder::new(EntityType::Fish, "fish", shape)
        .position(x, y, z)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .with_physics()
        .on_update(fish_update)
        .on_death(fish_death)
}

fn build_fish(screen_w: u16, screen_h: u16, classic: bool) -> EntityBuilder {
    build_fish_at(screen_w, screen_h, classic, false)
}

/// Spawn a fish scattered across the screen (for initial scene setup).
pub fn spawn_fish_scattered(app: &mut App, classic: bool) {
    let builder = build_fish_at(app.width, app.height, classic, true);
    app.spawn(builder);
}

/// Spawn a fish at the screen edge (for respawning after death).
pub fn spawn_fish(app: &mut App, classic: bool) {
    let builder = build_fish(app.width, app.height, classic);
    app.spawn(builder);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that for every fish definition, the art and mask have the same
    /// number of lines, and each corresponding line pair has the same length.
    fn validate_fish_defs(defs: &[FishDef], label: &str) {
        for (i, def) in defs.iter().enumerate() {
            for (dir, art, mask) in [
                ("right", def.right_art, def.right_mask),
                ("left", def.left_art, def.left_mask),
            ] {
                let art_lines: Vec<&str> = art.lines().collect();
                let mask_lines: Vec<&str> = mask.lines().collect();
                assert_eq!(
                    art_lines.len(),
                    mask_lines.len(),
                    "{} fish {} {}: art has {} lines but mask has {} lines",
                    label,
                    i,
                    dir,
                    art_lines.len(),
                    mask_lines.len(),
                );
                for (line_idx, (a, m)) in art_lines.iter().zip(mask_lines.iter()).enumerate() {
                    assert_eq!(
                        a.len(),
                        m.len(),
                        "{} fish {} {} line {}: art width {} != mask width {}\n  art:  {:?}\n  mask: {:?}",
                        label, i, dir, line_idx, a.len(), m.len(), a, m,
                    );
                }
            }
        }
    }

    #[test]
    fn old_fish_art_mask_dimensions_match() {
        validate_fish_defs(OLD_FISH_DEFS, "old");
    }

    #[test]
    fn new_fish_art_mask_dimensions_match() {
        validate_fish_defs(NEW_FISH_DEFS, "new");
    }

    #[test]
    fn extra_fish_art_mask_dimensions_match() {
        validate_fish_defs(EXTRA_FISH_DEFS, "extra");
    }

    #[test]
    fn all_fish_parse_without_panic() {
        let all: Vec<&FishDef> = OLD_FISH_DEFS
            .iter()
            .chain(NEW_FISH_DEFS.iter())
            .chain(EXTRA_FISH_DEFS.iter())
            .collect();
        for (i, def) in all.iter().enumerate() {
            // Parsing should not panic
            let _r = Shape::single(def.right_art, Some(def.right_mask), def.auto_trans);
            let _l = Shape::single(def.left_art, Some(def.left_mask), def.auto_trans);
            // Verify non-zero dimensions
            let rf = &_r.frames[0];
            assert!(
                rf.width > 0 && rf.height > 0,
                "fish {} right has zero dimensions",
                i
            );
            let lf = &_l.frames[0];
            assert!(
                lf.width > 0 && lf.height > 0,
                "fish {} left has zero dimensions",
                i
            );
        }
    }
}
