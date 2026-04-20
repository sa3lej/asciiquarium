use rand::Rng;

use crate::app::App;
use crate::color::Color;
use crate::entity::{
    DeathContext, EntityBuilder, EntityCommand, EntityType,
};
use crate::shape::Shape;

// ---------------------------------------------------------------------------
// Duck art — classic ASCII ducks, unmistakably duck-shaped
// ---------------------------------------------------------------------------

// Right-facing duck, frame 1
//  >(.)__
//   (____/
//    " "
const DUCK_RIGHT_1: &str = "\
 >(.)__
  (____/
   \" \"";

const DUCK_RIGHT_MASK_1: &str = "\
 YYYYY1
  YYYYY1
   B B";

// Right-facing duck, frame 2 (ripple shift)
const DUCK_RIGHT_2: &str = "\
 >(.)__
  (____/
  \" \"";

const DUCK_RIGHT_MASK_2: &str = "\
 YYYYY1
  YYYYY1
  B B";

// Left-facing duck, frame 1
//  __(.)<
//  \\____)
//    " "
const DUCK_LEFT_1: &str = "\
__(.)<
\\____)
  \" \"";

const DUCK_LEFT_MASK_1: &str = "\
1YYYYY
YYYYYY
  B B";

// Left-facing duck, frame 2 (ripple shift)
const DUCK_LEFT_2: &str = "\
__(.)<
\\____)
 \" \"";

const DUCK_LEFT_MASK_2: &str = "\
1YYYYY
YYYYYY
 B B";

// ---------------------------------------------------------------------------
// Duck spawning
// ---------------------------------------------------------------------------

fn build_duck_at(screen_w: u16, _screen_h: u16, scatter: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();

    let going_right = rng.gen_bool(0.5);

    let (frames, masks): (Vec<&str>, Vec<&str>) = if going_right {
        (
            vec![DUCK_RIGHT_1, DUCK_RIGHT_2],
            vec![DUCK_RIGHT_MASK_1, DUCK_RIGHT_MASK_2],
        )
    } else {
        (
            vec![DUCK_LEFT_1, DUCK_LEFT_2],
            vec![DUCK_LEFT_MASK_1, DUCK_LEFT_MASK_2],
        )
    };

    let frame_pairs: Vec<(&str, Option<&str>)> = frames
        .iter()
        .zip(masks.iter())
        .map(|(art, mask)| (*art, Some(*mask)))
        .collect();

    let shape = Shape::multi(frame_pairs, Some(' '));

    let frame = &shape.frames[0];
    let speed = rng.gen_range(0.3..0.9_f64);
    let vx = if going_right { speed } else { -speed };

    let x = if scatter {
        rng.gen_range(0.0..(screen_w as f64))
    } else if going_right {
        -(frame.width as f64)
    } else {
        screen_w as f64
    };

    // Ducks float ON the water surface. Water lines occupy rows 0-3.
    // Duck art is 3 rows tall; position so the beak/body sits at the
    // wave line with feet blending into the lower water rows.
    // y=1 places beak at row 1 (^^^), belly at row 2, feet at row 3.
    let y = 1.0;

    // z=1: in front of water-line segments (water z: 8,6,4,2 — lower z = closer to viewer)
    let z = 1;

    EntityBuilder::new(EntityType::Duck, "duck", shape)
        .position(x, y, z)
        .velocity(vx, 0.0)
        .color(Color::BrightYellow)
        .die_offscreen()
        .animate(0.5)
        .on_death(duck_death)
}

fn duck_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_duck_at(ctx.screen_width, ctx.screen_height, false);
    vec![EntityCommand::Spawn(builder)]
}

/// Spawn a duck at a random x position (for initial scene setup).
pub fn spawn_duck_scattered(app: &mut App) {
    let builder = build_duck_at(app.width, app.height, true);
    app.spawn(builder);
}
