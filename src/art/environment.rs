use rand::Rng;

use crate::app::App;
use crate::color::Color;
use crate::entity::{DeathContext, EntityBuilder, EntityCommand, EntityType};
use crate::shape::Shape;

// ---------------------------------------------------------------------------
// Water lines
// ---------------------------------------------------------------------------

const WATER_LINE_ART: &str = "\
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
^^^^ ^^^  ^^^   ^^^    ^^^^
^^^^      ^^^^     ^^^    ^^
^^      ^^^^      ^^^    ^^^^^^  ";

/// Spawn four water-line segments tiled across the full screen width.
/// Z depths: 8, 6, 4, 2 (each line is a separate entity so bubbles can
/// collide with the topmost one independently).
pub fn spawn_water(app: &mut App) {
    let z_depths: [i32; 4] = [8, 6, 4, 2];
    let lines: Vec<&str> = WATER_LINE_ART.lines().collect();
    let seg_width = lines[0].len() as u16;

    for (row, (&line, &z)) in lines.iter().zip(z_depths.iter()).enumerate() {
        // Tile the line to cover the screen width.
        let repeats = (app.width as usize / seg_width as usize) + 2;
        let tiled: String = line.repeat(repeats);
        // Trim to screen width
        let trimmed: String = tiled.chars().take(app.width as usize).collect();

        let shape = Shape::single(&trimmed, None, None);
        let builder = EntityBuilder::new(EntityType::Waterline, "waterline", shape)
            .position(0.0, row as f64, z)
            .color(Color::BrightBlue)
            .with_physics();
        app.spawn(builder);
    }
}

// ---------------------------------------------------------------------------
// Castle
// ---------------------------------------------------------------------------

// Frame 1: flag streams right, bright magenta glass, yellow windows
const CASTLE_ART_1: &str = "\
   >~]#[                  ]#[~~>
      ||                   ||
     /^^\\                 /^^\\                 /^^\\
     |  |                / ** \\                |  |
     |  |               / *{}* \\               |  |
     |  |             / * <||> * \\             |  |
 _   _   _   _   _ / *            * \\  _   _   _   _   _
[ ]_[ ]_[ ]_[ ]_[ ]_| *   <**>   * |  _[ ]_[ ]_[ ]_[ ]_[]
|_=-_=_-_=|=-_=_=-| *               * |=-_=_-=|_=-_=_=-_|
| _- =    | =_=_=-|  * *    **    * * |-_=_=  | _- =    |
|= -{ }   | -=_=-_|   * *  **  * *    |-=_=-  | -={ }   |
| =_      | =- =-_|  * *  <||>  * *   |=- -=  | =_      |
|= -{ }   | _=_=-_|   * *  **  * *    |-=_=-  | _={ }   |
| =_      | =- =-_|  *               *|=- -=  | =_      |
|= -{ }   | _=_=-_| -    /^^^^\\    -  |-=_=-  | -={ }   |
| =_      | =- =-_|  = * |####| * =   |=- -=  | =_      |
| = -     | _=_=-_|  = * |#  #| * =   |-=_=-  | =-      |
|- =_     | =- =-_|  =   |#  #|   =   |=- -=  |- =_     |
|_________|_______| _____|    |_______|_______|_________|";

const CASTLE_MASK_1: &str = "\
   RRRYR                  YRYRRR
      RR                   RR
     CccC                 CccC                 CccC
     c  c                C MM C                c  c
     c  c               C MYYM C               c  c
     c  c             C M YRRY M C             c  c
 c   c   c   c   c C M            M C  c   c   c   c   c
CcCcCcCcCcCcCcCcCcCcC M   RMMR   M C  cCcCcCcCcCcCcCcCcCC
cc c c c cc     c  C M               M C  c    cc c c c cc
c         c      cC  M M    MM    M M C       c         c
c   YY    c      cC   M M  MM  M M    C       c   YY    c
c         c      cC  M M  YRRY  M M   C       c         c
c   YY    c      cC   M M  MM  M M    C       c   YY    c
c         c      cC  M               MC       c         c
c   YY    c      cC       CccccC      C       c   YY    c
c         c      cC    Y yrrry Y      C       c         c
c         c      cC    Y yr  ry Y     C       c         c
c         c      cC      yr  ry       C       c         c
ccccccccccccccccccCcccccY    Yccccccc Ccccccccccccccccccc";

// Frame 2: flag flutters, dark magenta glass, green windows
const CASTLE_ART_2: &str = "\
    >]#[                  ]#[~>
      ||                   ||
     /^^\\                 /^^\\                 /^^\\
     |  |                / ** \\                |  |
     |  |               / *{}* \\               |  |
     |  |             / * <||> * \\             |  |
 _   _   _   _   _ / *            * \\  _   _   _   _   _
[ ]_[ ]_[ ]_[ ]_[ ]_| *   <**>   * |  _[ ]_[ ]_[ ]_[ ]_[]
|_=-_=_-_=|=-_=_=-| *               * |=-_=_-=|_=-_=_=-_|
| _- =    | =_=_=-|  * *    **    * * |-_=_=  | _- =    |
|= -{ }   | -=_=-_|   * *  **  * *    |-=_=-  | -={ }   |
| =_      | =- =-_|  * *  <||>  * *   |=- -=  | =_      |
|= -{ }   | _=_=-_|   * *  **  * *    |-=_=-  | _={ }   |
| =_      | =- =-_|  *               *|=- -=  | =_      |
|= -{ }   | _=_=-_| -    /^^^^\\    -  |-=_=-  | -={ }   |
| =_      | =- =-_|  = * |####| * =   |=- -=  | =_      |
| = -     | _=_=-_|  = * |#  #| * =   |-=_=-  | =-      |
|- =_     | =- =-_|  =   |#  #|   =   |=- -=  |- =_     |
|_________|_______| _____|    |_______|_______|_________|";

const CASTLE_MASK_2: &str = "\
    RRYR                  YRYRR
      RR                   RR
     CccC                 CccC                 CccC
     c  c                C mm C                c  c
     c  c               C mYYm C               c  c
     c  c             C m YRRY m C             c  c
 c   c   c   c   c C m            m C  c   c   c   c   c
CcCcCcCcCcCcCcCcCcCcC m   RmmR   m C  cCcCcCcCcCcCcCcCcCC
cc c c c cc     c  C m               m C  c    cc c c c cc
c         c      cC  m m    mm    m m C       c         c
c   yy    c      cC   m m  mm  m m    C       c   yy    c
c         c      cC  m m  YRRY  m m   C       c         c
c   yy    c      cC   m m  mm  m m    C       c   yy    c
c         c      cC  m               mC       c         c
c   yy    c      cC       CccccC      C       c   yy    c
c         c      cC    Y yrrry Y      C       c         c
c         c      cC    Y yr  ry Y     C       c         c
c         c      cC      yr  ry       C       c         c
ccccccccccccccccccCcccccY    Yccccccc Ccccccccccccccccccc";

pub fn spawn_castle(app: &mut App) {
    let shape = Shape::multi(
        vec![
            (CASTLE_ART_1, Some(CASTLE_MASK_1)),
            (CASTLE_ART_2, Some(CASTLE_MASK_2)),
        ],
        None,
    );
    // Position at bottom-right with enough margin so the castle is never clipped.
    let frame = &shape.frames[0];
    let x = (app.width as f64 - frame.width as f64 - 3.0).max(0.0);
    let y = app.height as f64 - frame.height as f64;
    let builder = EntityBuilder::new(EntityType::Castle, "castle", shape)
        .position(x, y, 22)
        .color(Color::White)
        .animate(0.5);
    app.spawn(builder);
}

// ---------------------------------------------------------------------------
// Seaweed
// ---------------------------------------------------------------------------

fn build_seaweed_shape(height: usize) -> Shape {
    let mut frame1 = String::new();
    let mut frame2 = String::new();

    for row in 0..height {
        if row % 2 == 0 {
            frame1.push_str("(");
            frame2.push_str(" )");
        } else {
            frame1.push_str(" )");
            frame2.push_str("(");
        }
        if row < height - 1 {
            frame1.push('\n');
            frame2.push('\n');
        }
    }

    let frames = vec![
        Frame::parse(&frame1, None, None),
        Frame::parse(&frame2, None, None),
    ];
    Shape { frames }
}

use crate::shape::Frame;

/// Compute the castle's left x boundary so seaweed can avoid it.
fn castle_left_x(screen_w: u16) -> u16 {
    let castle_w = 57u16; // max line width of CASTLE_ART
    (screen_w.saturating_sub(castle_w + 3)).max(5)
}

fn seaweed_death(_ctx: &DeathContext) -> Vec<EntityCommand> {
    let mut rng = rand::thread_rng();
    let height = rng.gen_range(3..=6);
    let shape = build_seaweed_shape(height);
    let anim_speed = rng.gen_range(0.25..=0.30);

    let screen_w = _ctx.screen_width;
    let screen_h = _ctx.screen_height;
    // Random x position, anchored to bottom, but avoid the castle zone
    let max_x = castle_left_x(screen_w);
    let x = rng.gen_range(5..max_x.max(6)) as f64;
    let y = (screen_h as f64) - (height as f64) - 1.0;

    let builder = EntityBuilder::new(EntityType::Seaweed, "seaweed", shape)
        .position(x, y, 21)
        .color(Color::BrightGreen)
        .animate(anim_speed)
        .on_death(seaweed_death);

    vec![EntityCommand::Spawn(builder)]
}

pub fn spawn_seaweed(app: &mut App) {
    let mut rng = rand::thread_rng();
    let height = rng.gen_range(3..=6);
    let shape = build_seaweed_shape(height);
    let anim_speed = rng.gen_range(0.25..=0.30);

    // Avoid spawning seaweed in the castle zone (bottom-right)
    let max_x = castle_left_x(app.width);
    let x = rng.gen_range(5..max_x.max(6)) as f64;
    let y = (app.height as f64) - (height as f64) - 1.0;

    let builder = EntityBuilder::new(EntityType::Seaweed, "seaweed", shape)
        .position(x, y, 21)
        .color(Color::BrightGreen)
        .animate(anim_speed)
        .on_death(seaweed_death);

    app.spawn(builder);
}
