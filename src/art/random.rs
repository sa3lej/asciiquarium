use rand::Rng;

use crate::app::App;
use crate::color::{self, Color};
use crate::entity::{
    DeathContext, Entity, EntityBuilder, EntityCommand, EntityType, UpdateContext,
};
use crate::shape::Shape;

// ---------------------------------------------------------------------------
// Random object spawner
// ---------------------------------------------------------------------------

/// Randomly picks one of the "big" creatures (shark, ship, whale, monster,
/// big_fish, hook) and spawns it.
pub fn spawn_random(app: &mut App, _classic: bool) {
    let mut rng = rand::thread_rng();
    // Dolphin and submarine are rare treats (~6% each)
    match rng.gen_range(0..16) {
        0 | 1 => spawn_shark(app),
        2 | 3 => spawn_ship(app),
        4 | 5 => spawn_whale(app),
        6 | 7 => spawn_monster(app),
        8 | 9 => spawn_big_fish(app),
        10 | 11 => spawn_hook(app),
        12 => spawn_dolphin(app),
        13 => spawn_submarine(app),
        _ => {
            // 14, 15: repeat common ones for weighted rarity
            match rng.gen_range(0..6) {
                0 => spawn_shark(app),
                1 => spawn_ship(app),
                2 => spawn_whale(app),
                3 => spawn_monster(app),
                4 => spawn_big_fish(app),
                _ => spawn_hook(app),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shark
// ---------------------------------------------------------------------------

const SHARK_RIGHT_ART: &str = "\
                              __
                             ( `\\
  ,--------,----------,--------|  |
 /          '.        `        |  |
(             `.     ,-------- / _/
 \\             /    /  /    .-'    '---.
  `--------' /    /  /   .'    '.      )
             (   (  (   /       /   .-'
              '-' '-' '-'      '-'";

const SHARK_RIGHT_MASK: &str = "\
                              11
                             1 11
  1111111111111111111111111111  1
 1          11        1        1
1             11     11111111 1 1
 1             1    1  1    111    1111
  111111111 1    1  1   11    11      1
             1   1  1   1       1   11
              111 111 111      111";

const SHARK_LEFT_ART: &str = "\
 __
/` )
|  |--------,----------,--------,
|  |        '        .'          \\
\\_  \\ --------,     .'             )
 .---'    .-'  \\    \\             /
(      .'    '.   \\    \\ '--------'
  '-.   \\       )   )   )
      '-'      '-' '-' '-'";

const SHARK_LEFT_MASK: &str = "\
 11
11 1
1  1111111111111111111111111111
1        1        11          1
1 1 11111111     11             1
 1111    1  1    1             1
1      11    11   1    1 111111111
  11   1       1   1   1
      111      111 111 111";

const TEETH_RIGHT_ART: &str = "\
 ^  ^     ^
/|\\/ \\   / \\";

const TEETH_LEFT_ART: &str = "\
    ^     ^  ^
   / \\   / \\/|\\";

fn shark_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn shark_update(_entity: &mut Entity, _ctx: &UpdateContext) -> Vec<EntityCommand> {
    // Shark doesn't do much beyond moving (handled by physics).
    vec![]
}

fn spawn_shark(app: &mut App) {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);

    let (art, mask_str, teeth_art) = if going_right {
        (SHARK_RIGHT_ART, SHARK_RIGHT_MASK, TEETH_RIGHT_ART)
    } else {
        (SHARK_LEFT_ART, SHARK_LEFT_MASK, TEETH_LEFT_ART)
    };

    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), Some(' '));

    let frame = &shape.frames[0];
    let speed = rng.gen_range(1.0..3.0_f64);
    let vx = if going_right { speed } else { -speed };

    let x = if going_right {
        -(frame.width as f64)
    } else {
        app.width as f64
    };

    let min_y = 5.0;
    let max_y = (app.height as f64) - (frame.height as f64) - 2.0;
    let y = rng.gen_range(min_y..max_y.max(min_y + 1.0));
    let z = 18;

    let shark_builder = EntityBuilder::new(EntityType::Shark, "shark", shape)
        .position(x, y, z)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .on_update(shark_update)
        .on_death(shark_death);

    let shark_id = app.spawn(shark_builder);

    // Spawn teeth entity that follows below the shark mouth.
    let teeth_shape = Shape::single(teeth_art, None, None);
    let teeth_x = if going_right {
        x + 2.0
    } else {
        x + 2.0
    };
    let teeth_y = y + 5.0; // below the shark body

    let mut teeth_builder = EntityBuilder::new(EntityType::Teeth, "teeth", teeth_shape)
        .position(teeth_x, teeth_y, z - 1)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .with_physics();
    teeth_builder.extra.follow_entity = Some(shark_id);
    app.spawn(teeth_builder);
}

// ---------------------------------------------------------------------------
// Ship
// ---------------------------------------------------------------------------

const SHIP_RIGHT_ART: &str = "\
     |    |
    )_____|____
    )          |\\
    )   ____   | |
    )__/    \\__| |
   )|          | |
    )__________| |
        |______|/";

const SHIP_RIGHT_MASK: &str = "\
     1    1
    111111111
    1          11
    1   1111   1 1
    1111    1111 1
   11          1 1
    11111111111 1
        11111111";

const SHIP_LEFT_ART: &str = "\
   |    |
 ____|_____(
/|          (
| |   ____  (
| |__/    \\_(
| |          |)
| |__________(
 \\|______|";

const SHIP_LEFT_MASK: &str = "\
   1    1
 1111111111
11          1
1 1   1111  1
1 1111    1111
1 1          11
1 11111111111
 11111111";

fn ship_death(_ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(_ctx.screen_width, _ctx.screen_height, _ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn spawn_ship(app: &mut App) {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);

    let (art, mask_str) = if going_right {
        (SHIP_RIGHT_ART, SHIP_RIGHT_MASK)
    } else {
        (SHIP_LEFT_ART, SHIP_LEFT_MASK)
    };

    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), Some(' '));

    let frame = &shape.frames[0];
    let speed = rng.gen_range(0.5..1.5_f64);
    let vx = if going_right { speed } else { -speed };

    let x = if going_right {
        -(frame.width as f64)
    } else {
        app.width as f64
    };

    // Ship rides on the water surface.
    let y = 0.0;
    let z = 7;

    let builder = EntityBuilder::new(EntityType::Ship, "ship", shape)
        .position(x, y, z)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .on_death(ship_death);

    app.spawn(builder);
}

// ---------------------------------------------------------------------------
// Whale
// ---------------------------------------------------------------------------

// Whale body images (right=dir 0, left=dir 1 in the Perl source).
// Leading \n mirrors the Perl q{} string that starts with a newline.
const WHALE_BODIES: [&str; 2] = [
    // Right-facing (dir=0)
    "\n\
        .-----:\n\
      .'       `.\n\
,????/       (o) \\\n\
\\`._/          ,__)\n",
    // Left-facing (dir=1)
    "\n\
    :-----.\n\
  .'       `.\n\
 / (o)       \\????,\n\
(__,          \\_.'/\n",
];

// Water spout animation frames (7 frames).
// Each string mirrors the Perl q{} content including leading/trailing \n.
const WATER_SPOUT: [&str; 7] = [
    "\n\n\n   :\n",
    "\n\n   :\n   :\n",
    "\n  . .\n  -:-\n   :\n",
    "\n  . .\n .-:-.\n   :\n",
    "\n  . .\n'.-:-.`\n'  :  '\n",
    "\n\n .- -.\n;  :  ;\n",
    "\n\n\n;     ;\n",
];

// Whale color masks.
const WHALE_MASKS: [&str; 2] = [
    // Right-facing mask
    "\n\
             C C\n\
           CCCCCCC\n\
           C  C  C\n\
        BBBBBBB\n\
      BB       BB\n\
B    B       BWB B\n\
BBBBB          BBBB\n",
    // Left-facing mask
    "\n\
   C C\n\
 CCCCCCC\n\
 C  C  C\n\
    BBBBBBB\n\
  BB       BB\n\
 B BWB       B    B\n\
BBBB          BBBBB\n",
];

/// Replicate the Perl spout alignment logic:
///   split spout on "\n", join with "\n" + " " * spout_align, concat with whale body.
fn build_whale_spout_frame(spout: &str, spout_align: usize, body: &str) -> String {
    let parts: Vec<&str> = spout.split('\n').collect();
    // Perl split removes trailing empty elements
    let trimmed: Vec<&str> = {
        let mut v = parts;
        while v.last() == Some(&"") {
            v.pop();
        }
        v
    };
    let indent: String = " ".repeat(spout_align);
    let sep = format!("\n{}", indent);
    let aligned = trimmed.join(&sep);
    format!("{}{}", aligned, body)
}

fn whale_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

/// Build the whale Shape with all animation frames (5 no-spout + 7 spout).
fn make_whale_shape(dir: usize) -> Shape {
    let body = WHALE_BODIES[dir];
    let mask = WHALE_MASKS[dir];
    // spout_align: right-facing(dir=0)=11, left-facing(dir=1)=1
    let spout_align = if dir == 0 { 11 } else { 1 };

    let mut frame_strings: Vec<(String, String)> = Vec::with_capacity(12);

    // 5 frames with no water spout
    let no_spout = format!("\n\n\n{}", body);
    for _ in 0..5 {
        frame_strings.push((no_spout.clone(), mask.to_string()));
    }

    // 7 frames with water spout animation
    for spout in &WATER_SPOUT {
        let combined = build_whale_spout_frame(spout, spout_align, body);
        frame_strings.push((combined, mask.to_string()));
    }

    let frame_refs: Vec<(&str, Option<&str>)> = frame_strings
        .iter()
        .map(|(art, m)| (art.as_str(), Some(m.as_str())))
        .collect();

    Shape::multi(frame_refs, Some('?'))
}

fn spawn_whale(app: &mut App) {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);
    let dir = if going_right { 0 } else { 1 };

    let shape = make_whale_shape(dir);

    let speed = 1.0_f64;
    let vx = if going_right { speed } else { -speed };

    let first_frame = &shape.frames[0];
    let x = if going_right {
        -(first_frame.width as f64)
    } else {
        app.width as f64
    };

    let builder = EntityBuilder::new(EntityType::Whale, "whale", shape)
        .position(x, 0.0, 5)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .animate(1.0)
        .on_death(whale_death);

    app.spawn(builder);
}

fn build_whale(screen_w: u16, _screen_h: u16, going_right: bool) -> EntityBuilder {
    let dir = if going_right { 0 } else { 1 };
    let shape = make_whale_shape(dir);

    let speed = 1.0_f64;
    let vx = if going_right { speed } else { -speed };

    let first_frame = &shape.frames[0];
    let x = if going_right {
        -(first_frame.width as f64)
    } else {
        screen_w as f64
    };

    EntityBuilder::new(EntityType::Whale, "whale", shape)
        .position(x, 0.0, 5)
        .velocity(vx, 0.0)
        .color(Color::White)
        .die_offscreen()
        .animate(1.0)
        .on_death(whale_death)
}

// ---------------------------------------------------------------------------
// Monster (new style, 2 frames)
// ---------------------------------------------------------------------------

// Right-facing monster, frame 0
const MONSTER_RIGHT_ART_0: &str = "\
\n\
         _???_?????????????????????_???_???????_a_a\n\
       _{.`=`.}_??????_???_??????_{.`=`.}_????{/ ''\\_\n\
 _????{.'  _  '.}????{.`'`.}????{.'  _  '.}??{|  ._oo)\n\
{ \\??{/  .'?'.  \\}??{/ .-. \\}??{/  .'?'.  \\}?{/  |";

// Right-facing monster, frame 1
const MONSTER_RIGHT_ART_1: &str = "\
\n\
                      _???_????????????????????_a_a\n\
  _??????_???_??????_{.`=`.}_??????_???_??????{/ ''\\_\n\
 { \\????{.`'`.}????{.'  _  '.}????{.`'`.}????{|  ._oo)\n\
  \\ \\??{/ .-. \\}??{/  .'?'.  \\}??{/ .-. \\}???{/  |";

// Left-facing monster, frame 0
const MONSTER_LEFT_ART_0: &str = "\
\n\
   a_a_???????_???_?????????????????????_???_\n\
 _/'' \\}????_{.`=`.}_??????_???_??????_{.`=`.}_\n\
(oo_.  |}??{.'  _  '.}????{.`'`.}????{.'  _  '.}????_\n\
    |  \\}?{/  .'?'.  \\}??{/ .-. \\}??{/  .'?'.  \\}??/ }";

// Left-facing monster, frame 1
const MONSTER_LEFT_ART_1: &str = "\
\n\
   a_a_????????????????????_   _\n\
 _/'' \\}??????_???_??????_{.`=`.}_??????_???_??????_\n\
(oo_.  |}????{.`'`.}????{.'  _  '.}????{.`'`.}????/ }\n\
    |  \\}???{/ .-. \\}??{/  .'?'.  \\}??{/ .-. \\}??/ /";

// Monster color masks. W marks the eyes.
const MONSTER_RIGHT_MASK: &str = "\
                                                W W\n\
\n\
\n\
\n\
";

const MONSTER_LEFT_MASK: &str = "\
\n\
   W W\n\
\n\
\n\
";

// ---------------------------------------------------------------------------
// Old monster (4-frame animation, larger)
// ---------------------------------------------------------------------------

// Right-facing old monster, 4 frames
const OLD_MONSTER_RIGHT_0: &str = "\
                                                          ____\n\
            __??????????????????????????????????????????/   o  \\\n\
          /    \\????????_?????????????????????_???????/     ____ >\n\
  _??????|  __  |?????/   \\????????_????????/   \\????|     |\n\
 | \\?????|  ||  |????|     |?????/   \\?????|     |???|     |";

const OLD_MONSTER_RIGHT_1: &str = "\
                                                          ____\n\
                                             __?????????/   o  \\\n\
             _?????????????????????_???????/    \\?????/     ____ >\n\
   _???????/   \\????????_????????/   \\????|  __  |???|     |\n\
  | \\?????|     |?????/   \\?????|     |???|  ||  |???|     |";

const OLD_MONSTER_RIGHT_2: &str = "\
                                                          ____\n\
                                  __????????????????????/   o  \\\n\
 _??????????????????????_???????/    \\????????_???????/     ____ >\n\
| \\??????????_????????/   \\????|  __  |?????/   \\????|     |\n\
 \\ \\???????/   \\?????|     |???|  ||  |????|     |???|     |";

const OLD_MONSTER_RIGHT_3: &str = "\
                                                          ____\n\
                       __???????????????????????????????/   o  \\\n\
  _??????????_???????/    \\????????_??????????????????/     ____ >\n\
 | \\???????/   \\????|  __  |?????/   \\????????_??????|     |\n\
  \\ \\?????|     |???|  ||  |????|     |?????/   \\????|     |";

// Left-facing old monster, 4 frames
const OLD_MONSTER_LEFT_0: &str = "\
    ____\n\
  /  o   \\??????????????????????????????????????????__\n\
< ____     \\???????_?????????????????????_????????/    \\\n\
      |     |????/   \\????????_????????/   \\?????|  __  |??????_\n\
      |     |???|     |?????/   \\?????|     |????|  ||  |?????/ |";

const OLD_MONSTER_LEFT_1: &str = "\
    ____\n\
  /  o   \\?????????__\n\
< ____     \\?????/    \\???????_?????????????????????_\n\
      |     |???|  __  |????/   \\????????_????????/   \\???????_\n\
      |     |???|  ||  |???|     |?????/   \\?????|     |?????/ |";

const OLD_MONSTER_LEFT_2: &str = "\
    ____\n\
  /  o   \\????????????????????__\n\
< ____     \\???????_????????/    \\???????_??????????????????????_\n\
      |     |????/   \\?????|  __  |????/   \\????????_??????????/ |\n\
      |     |???|     |????|  ||  |???|     |?????/   \\???????/ /";

const OLD_MONSTER_LEFT_3: &str = "\
    ____\n\
  /  o   \\???????????????????????????????__\n\
< ____     \\??????????????????_????????/    \\???????_??????????_\n\
      |     |??????_????????/   \\?????|  __  |????/   \\???????/ |\n\
      |     |????/   \\?????|     |????|  ||  |???|     |?????/ /";

const OLD_MONSTER_RIGHT_MASK: &str = "\
\n\
                                                            W\n\
\n\
\n\
";

const OLD_MONSTER_LEFT_MASK: &str = "\
\n\
     W\n\
\n\
\n\
";

fn monster_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn spawn_monster(app: &mut App) {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);
    let use_old = rng.gen_bool(0.5);

    let shape = if use_old {
        build_old_monster_shape(going_right)
    } else {
        build_new_monster_shape(going_right)
    };

    let speed = 2.0_f64;
    let vx = if going_right { speed } else { -speed };

    let first_frame = &shape.frames[0];
    let x = if going_right {
        -(first_frame.width as f64)
    } else {
        app.width as f64
    };

    let builder = EntityBuilder::new(EntityType::Monster, "monster", shape)
        .position(x, 2.0, 5)
        .velocity(vx, 0.0)
        .color(Color::Green)
        .die_offscreen()
        .animate(0.25)
        .on_death(monster_death);

    app.spawn(builder);
}

fn build_new_monster_shape(going_right: bool) -> Shape {
    let (art_0, art_1, mask_str) = if going_right {
        (MONSTER_RIGHT_ART_0, MONSTER_RIGHT_ART_1, MONSTER_RIGHT_MASK)
    } else {
        (MONSTER_LEFT_ART_0, MONSTER_LEFT_ART_1, MONSTER_LEFT_MASK)
    };
    let frames: Vec<(&str, Option<&str>)> = vec![
        (art_0, Some(mask_str)),
        (art_1, Some(mask_str)),
    ];
    Shape::multi(frames, Some('?'))
}

fn build_old_monster_shape(going_right: bool) -> Shape {
    let (arts, mask_str) = if going_right {
        (
            [OLD_MONSTER_RIGHT_0, OLD_MONSTER_RIGHT_1, OLD_MONSTER_RIGHT_2, OLD_MONSTER_RIGHT_3],
            OLD_MONSTER_RIGHT_MASK,
        )
    } else {
        (
            [OLD_MONSTER_LEFT_0, OLD_MONSTER_LEFT_1, OLD_MONSTER_LEFT_2, OLD_MONSTER_LEFT_3],
            OLD_MONSTER_LEFT_MASK,
        )
    };
    let frames: Vec<(&str, Option<&str>)> = arts
        .iter()
        .map(|art| (*art, Some(mask_str)))
        .collect();
    Shape::multi(frames, Some('?'))
}

fn build_monster(screen_w: u16, _screen_h: u16, going_right: bool) -> EntityBuilder {
    let use_old = rand::thread_rng().gen_bool(0.5);
    let shape = if use_old {
        build_old_monster_shape(going_right)
    } else {
        build_new_monster_shape(going_right)
    };

    let speed = 2.0_f64;
    let vx = if going_right { speed } else { -speed };

    let first_frame = &shape.frames[0];
    let x = if going_right {
        -(first_frame.width as f64)
    } else {
        screen_w as f64
    };

    EntityBuilder::new(EntityType::Monster, "monster", shape)
        .position(x, 2.0, 5)
        .velocity(vx, 0.0)
        .color(Color::Green)
        .die_offscreen()
        .animate(0.25)
        .on_death(monster_death)
}

// ---------------------------------------------------------------------------
// Big Fish (variant 1)
// ---------------------------------------------------------------------------

const BIG_FISH_1_RIGHT_ART: &str = "\
 ______\n\
`\"\"-.  `````-----.....__\n\
     `.  .      .       `-.\n\
       :     .     .       `.\n\
 ,?????:   .    .          _ :\n\
: `.???:                  (@) `._\n\
 `. `..'     .     =`-.       .__)\n\
   ;     .        =  ~  :     .-\"\n\
 .' .'`.   .    .  =.-'  `._ .'\n\
: .'???:               .   .'\n\
 '???.'  .    .     .   .-'\n\
   .'____....----''.'=.'\n\
   \"\"?????????????.'.' \n\
               ''\"'`";

const BIG_FISH_1_RIGHT_MASK: &str = "\
 111111\n\
11111  11111111111111111\n\
     11  2      2       111\n\
       1     2     2       11\n\
 1     1   2    2          1 1\n\
1 11   1                  1W1 111\n\
 11 1111     2     1111       1111\n\
   1     2        1  1  1     111\n\
 11 1111   2    2  1111  111 11\n\
1 11   1               2   11\n\
 1   11  2    2     2   111\n\
   111111111111111111111\n\
   11             1111\n\
               11111";

const BIG_FISH_1_LEFT_ART: &str = "\
                           ______\n\
          __.....-----'''''  .-\"\"'\n\
       .-'       .      .  .'\n\
     .'       .     .     :\n\
    : _          .    .   :?????,\n\
 _.' (@)                  :???.' :\n\
(__.       .-'=     .     `..' .'\n\
 \"-.     :  ~  =        .     ;\n\
   `. _.'  `-.=  .    .   .'`. `.\n\
     `.   .               :???`. :\n\
       `-.   .     .    .  `.???`\n\
          `.=`.``----....____`.\n\
            `.`.?????????????\"\"\n\
              '`\"``";

const BIG_FISH_1_LEFT_MASK: &str = "\
                           111111\n\
          11111111111111111  11111\n\
       111       2      2  11\n\
     11       2     2     1\n\
    1 1          2    2   1     1\n\
 111 1W1                  1   11 1\n\
1111       1111     2     1111 11\n\
 111     1  1  1        2     1\n\
   11 111  1111  2    2   1111 11\n\
     11   2               1   11 1\n\
       111   2     2    2  11   1\n\
          111111111111111111111\n\
            1111             11\n\
              11111";

// ---------------------------------------------------------------------------
// Big Fish (variant 2)
// ---------------------------------------------------------------------------

const BIG_FISH_2_RIGHT_ART: &str = "\
                _ _ _\n\
             .='\\ \\ \\`\"=,\n\
           .'\\ \\ \\ \\ \\ \\ \\\n\
\\'=._?????/ \\ \\ \\_\\_\\_\\_\\_\\\n\
\\'=._'.??/\\ \\,-\"`- _ - _ - '-.\n\
  \\`=._\\|'.\\/- _ - _ - _ - _- \\\n\
  ;\"= ._\\=./_ -_ -_ {`\"=_    @ \\\n\
   ;\"=_-_=- _ -  _ - {\"=_\"-     \\\n\
   ;_=_--_.,          {_.='   .-/\n\
  ;.=\"` / ';\\        _.     _.-`\n\
  /_.='/ \\/ /;._ _ _{.-;`/\"`\n\
/._=_.'???'/ / / / /{.= /\n\
/.='???????`'./_/_.=`{_/";

const BIG_FISH_2_RIGHT_MASK: &str = "\
                1 1 1\n\
             1111 1 11111\n\
           111 1 1 1 1 1 1\n\
11111     1 1 1 11111111111\n\
1111111  11 111112 2 2 2 2 111\n\
  111111111112 2 2 2 2 2 2 22 1\n\
  111 1111 12 22 22 11111    W 1\n\
   11111112 2 2  2 2 111111     1\n\
   111111111          11111   111\n\
  11111 11111        11     1111\n\
  111111 11 1111 1 111111111\n\
1111111   11 1 1 1 1111 1\n\
1111       1111111111111";

const BIG_FISH_2_LEFT_ART: &str = "\
            _ _ _\n\
        ,=\"`/ / /'=.\n\
       / / / / / / /'.\n\
      /_/_/_/_/_/ / / \\?????_.='/\n\
   .-' - _ - _ -`\"-,/ /\\??.'_.='/\n\
  / -_ - _ - _ - _ -\\/.'|/_.=`/\n\
 / @    _=\"`} _- _- _\\.=/_. =\";\n\
/     -\"_=\"} - _  - _ -=_-_\"=;\n\
\\-.   '=._}          ,._--_=_;\n\
 `-._     ._        /;' \\ `\"=.;\n\
     `\"\\`;-.}_ _ _.;\\ \\/ \\'=._\\\n\
        \\ =.}\\ \\ \\ \\ \\'???'._=_.\\\n\
         \\_}`=._\\_\\.'`???????'=.\\";

const BIG_FISH_2_LEFT_MASK: &str = "\
            1 1 1\n\
        11111 1 1111\n\
       1 1 1 1 1 1 111\n\
      11111111111 1 1 1     11111\n\
   111 2 2 2 2 211111 11  1111111\n\
  1 22 2 2 2 2 2 2 211111111111\n\
 1 W    11111 22 22 2111111 111\n\
1     111111 2 2  2 2 21111111\n\
111   11111          111111111\n\
 1111     11        111 1 11111\n\
     111111111 1 1111 11 111111\n\
        1 1111 1 1 1 11   1111111\n\
         1111111111111       1111";

fn big_fish_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn spawn_big_fish(app: &mut App) {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);

    // Pick variant 1 or 2 (2/3 chance of variant 2, 1/3 of variant 1)
    let variant = if rng.gen_range(0..3) > 0 { 2 } else { 1 };

    let (art, mask_str, speed) = match (variant, going_right) {
        (1, true) => (BIG_FISH_1_RIGHT_ART, BIG_FISH_1_RIGHT_MASK, 3.0_f64),
        (1, false) => (BIG_FISH_1_LEFT_ART, BIG_FISH_1_LEFT_MASK, 3.0_f64),
        (_, true) => (BIG_FISH_2_RIGHT_ART, BIG_FISH_2_RIGHT_MASK, 2.5_f64),
        (_, false) => (BIG_FISH_2_LEFT_ART, BIG_FISH_2_LEFT_MASK, 2.5_f64),
    };

    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), Some('?'));

    let frame = &shape.frames[0];
    let vx = if going_right { speed } else { -speed };

    let x = if going_right {
        -(frame.width as f64)
    } else {
        app.width as f64
    };

    let min_y = 9.0;
    let max_y = (app.height as f64) - (frame.height as f64) - 1.0;
    let y = rng.gen_range(min_y..max_y.max(min_y + 1.0));
    let z = 18; // shark depth in Rust

    let builder = EntityBuilder::new(EntityType::BigFish, "big_fish", shape)
        .position(x, y, z)
        .velocity(vx, 0.0)
        .color(Color::Yellow)
        .die_offscreen()
        .on_death(big_fish_death);

    app.spawn(builder);
}

fn build_big_fish(screen_w: u16, screen_h: u16, going_right: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();
    let variant = if rng.gen_range(0..3) > 0 { 2 } else { 1 };

    let (art, mask_str, speed) = match (variant, going_right) {
        (1, true) => (BIG_FISH_1_RIGHT_ART, BIG_FISH_1_RIGHT_MASK, 3.0_f64),
        (1, false) => (BIG_FISH_1_LEFT_ART, BIG_FISH_1_LEFT_MASK, 3.0_f64),
        (_, true) => (BIG_FISH_2_RIGHT_ART, BIG_FISH_2_RIGHT_MASK, 2.5_f64),
        (_, false) => (BIG_FISH_2_LEFT_ART, BIG_FISH_2_LEFT_MASK, 2.5_f64),
    };

    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), Some('?'));

    let frame = &shape.frames[0];
    let vx = if going_right { speed } else { -speed };

    let x = if going_right {
        -(frame.width as f64)
    } else {
        screen_w as f64
    };

    let min_y = 9.0;
    let max_y = (screen_h as f64) - (frame.height as f64) - 1.0;
    let y = rng.gen_range(min_y..max_y.max(min_y + 1.0));

    EntityBuilder::new(EntityType::BigFish, "big_fish", shape)
        .position(x, y, 18)
        .velocity(vx, 0.0)
        .color(Color::Yellow)
        .die_offscreen()
        .on_death(big_fish_death)
}

// ---------------------------------------------------------------------------
// Helper: build a random entity (used from death callbacks that can't access App)
// ---------------------------------------------------------------------------

fn build_random(screen_w: u16, screen_h: u16, _classic: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();
    let going_right = rng.gen_bool(0.5);

    // Dolphin (5) and submarine (6) are rare
    match rng.gen_range(0..8) {
        0 => {
            // Build a shark builder
            let (art, mask_str) = if going_right {
                (SHARK_RIGHT_ART, SHARK_RIGHT_MASK)
            } else {
                (SHARK_LEFT_ART, SHARK_LEFT_MASK)
            };
            let colored_mask = color::rand_color(mask_str);
            let shape = Shape::single(art, Some(&colored_mask), Some(' '));
            let frame = &shape.frames[0];
            let speed = rng.gen_range(1.0..3.0_f64);
            let vx = if going_right { speed } else { -speed };
            let x = if going_right {
                -(frame.width as f64)
            } else {
                screen_w as f64
            };
            let min_y = 5.0;
            let max_y = (screen_h as f64) - 12.0;
            let y = rng.gen_range(min_y..max_y.max(min_y + 1.0));

            EntityBuilder::new(EntityType::Shark, "shark", shape)
                .position(x, y, 18)
                .velocity(vx, 0.0)
                .color(Color::White)
                .die_offscreen()
                .on_update(shark_update)
                .on_death(shark_death)
        }
        1 => {
            // Build a ship builder
            let (art, mask_str) = if going_right {
                (SHIP_RIGHT_ART, SHIP_RIGHT_MASK)
            } else {
                (SHIP_LEFT_ART, SHIP_LEFT_MASK)
            };
            let colored_mask = color::rand_color(mask_str);
            let shape = Shape::single(art, Some(&colored_mask), Some(' '));
            let frame = &shape.frames[0];
            let speed = rng.gen_range(0.5..1.5_f64);
            let vx = if going_right { speed } else { -speed };
            let x = if going_right {
                -(frame.width as f64)
            } else {
                screen_w as f64
            };

            EntityBuilder::new(EntityType::Ship, "ship", shape)
                .position(x, 0.0, 7)
                .velocity(vx, 0.0)
                .color(Color::White)
                .die_offscreen()
                .on_death(ship_death)
        }
        2 => build_whale(screen_w, screen_h, going_right),
        3 => build_monster(screen_w, screen_h, going_right),
        4 => build_big_fish(screen_w, screen_h, going_right),
        5 => build_dolphin(screen_w, screen_h, going_right),
        6 => build_submarine(screen_w, screen_h, going_right),
        _ => build_big_fish(screen_w, screen_h, going_right),
    }
}

// ---------------------------------------------------------------------------
// Fishhook — based on the community asciiquarium 1.2 design
// ---------------------------------------------------------------------------

// The hook sprite — clear, visible hook shape:
//       o        <- eyelet / swivel
//      ||        <- shank (double line)
//  .   ||        <- barb tip
// /'\  ||        <- barb curve
//  \___/         <- bend
//   `--'         <- bottom
const HOOK_ART: &str = "\
      o\n\
     ||\n\
 .   ||\n\
/'\\  ||\n\
 \\___/\n\
  `--'";

const HOOK_MASK: &str = "\
      W\n\
     WW\n\
 W   WW\n\
WWW  WW\n\
 WWWWW\n\
  WWWW";

// Fishing rod — sits at the water surface while the line is in the water
// The rod tip bends down where the line attaches:
//  \\
//   \\___,--.
//        |
const ROD_ART: &str = "\
 \\\n\
  \\___,--.";

const ROD_MASK: &str = "\
 Y\n\
  YYYYYYW";

/// Build the complete fishing line + hook shape.
/// A column of `||` extends above the hook (offscreen),
/// positioned to align with the hook's shank.
const LINE_ROWS: usize = 50;

fn build_hook_shape() -> Shape {
    let mut art = String::new();
    let mut mask = String::new();
    // Build fishing line — || at column 5-6 to align with the hook shank
    for _ in 0..LINE_ROWS {
        art.push_str("     ||\n");
        mask.push_str("     WW\n");
    }
    // Append the hook itself
    art.push_str(HOOK_ART);
    mask.push_str(HOOK_MASK);

    Shape::single(&art, Some(&mask), Some(' '))
}

/// Build the fishing rod shape (surface bobber/rod tip).
fn build_rod_shape() -> Shape {
    Shape::single(ROD_ART, Some(ROD_MASK), Some(' '))
}

/// Hook states encoded in velocity:
///   vy > 0  => lowering (descending into water)
///   vy == 0 => dwelling at depth (timer via frame_timer)
///   vy < 0  => retracting (reeling back up)
const HOOK_SPEED: f64 = 8.0;
const HOOK_DWELL_TIME: f64 = 15.0;
const HOOK_HEIGHT: usize = LINE_ROWS + 6; // line rows + hook rows

fn hook_update(entity: &mut Entity, ctx: &UpdateContext) -> Vec<EntityCommand> {
    let max_depth = (ctx.screen_height as f64) * 0.75 - HOOK_HEIGHT as f64;
    let mut cmds = vec![];

    if entity.vy > 0.0 {
        // Lowering — spawn rod at surface when line first becomes visible
        // The line top is at entity.y; it becomes visible when entity.y > -LINE_ROWS as f64
        // Spawn the rod once, when the line tip crosses the water surface (row ~2)
        if !entity.extra.rod_spawned {
            let visible_top = entity.y + LINE_ROWS as f64;
            if visible_top >= 1.0 {
                // Place rod at the water surface, aligned with the line
                // Rod tip points right, line drops from the right end
                // The || of the line is at column 5-6 relative to hook entity x.
                // Rod art is 2 rows; place it so the rod end aligns with the line.
                let rod_x = entity.x; // rod starts a few cols left of the line
                let rod = EntityBuilder::new(EntityType::RandomObject, "fishing_rod", build_rod_shape())
                    .position(rod_x, 1.0, 1) // z=1: in front of water
                    .color(Color::BrightYellow);
                cmds.push(EntityCommand::Spawn(rod));
                entity.extra.rod_spawned = true;
            }
        }

        // Check if the hook tip reached target depth
        if entity.y >= max_depth {
            entity.vy = 0.0;
            entity.frame_timer = 0.0; // reuse as dwell timer
        }
    } else if entity.vy == 0.0 {
        // Dwelling — count time at depth
        entity.frame_timer += ctx.dt;
        if entity.frame_timer >= HOOK_DWELL_TIME {
            entity.vy = -HOOK_SPEED; // start retracting
        }
    } else {
        // Retracting — check if fully offscreen above
        if entity.y < -60.0 {
            return vec![EntityCommand::KillSelf];
        }
    }
    cmds
}

fn hook_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    // When the hook is done, spawn a new random object
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

// ---------------------------------------------------------------------------
// Dolphin — leaping pair near the surface
// ---------------------------------------------------------------------------

// Right-facing dolphin, 2 animation frames (tail flick)
// All frames are consistently 13 chars wide, 3 lines tall
const DOLPHIN_RIGHT_FRAMES: [&str; 2] = [
    "\
     ,---.  \n\
>~_-'  o }=>\n\
    `---'   ",
    "\
     ,---.  \n\
>~_-'  o }~>\n\
    `---'   ",
];

const DOLPHIN_LEFT_FRAMES: [&str; 2] = [
    "\
  .---.     \n\
<={ o  '-_~<\n\
   '---'    ",
    "\
  .---.     \n\
<~{ o  '-_~<\n\
   '---'    ",
];

const DOLPHIN_RIGHT_MASK: &str = "\
     CCCC\n\
CCCCC  W CCC\n\
    CCCCC";

const DOLPHIN_LEFT_MASK: &str = "\
    CCCC\n\
CCC W  CCCCC\n\
  CCCCC";

fn build_dolphin(screen_w: u16, _screen_h: u16, going_right: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();

    let frames = if going_right {
        &DOLPHIN_RIGHT_FRAMES
    } else {
        &DOLPHIN_LEFT_FRAMES
    };
    let mask = if going_right {
        DOLPHIN_RIGHT_MASK
    } else {
        DOLPHIN_LEFT_MASK
    };

    let colored_mask = color::rand_color(mask);

    let frame_pairs: Vec<(&str, Option<&str>)> = frames
        .iter()
        .map(|art| (*art, Some(colored_mask.as_str())))
        .collect();

    let shape = Shape::multi(frame_pairs, Some(' '));

    let first_frame = &shape.frames[0];
    let speed = rng.gen_range(1.5..2.5_f64);
    let vx = if going_right { speed } else { -speed };
    let x = if going_right {
        -(first_frame.width as f64)
    } else {
        screen_w as f64
    };

    // Dolphins leap near the surface
    let y = 2.0;

    EntityBuilder::new(EntityType::Dolphin, "dolphin", shape)
        .position(x, y, 5)
        .velocity(vx, 0.0)
        .color(Color::Cyan)
        .die_offscreen()
        .animate(0.5)
        .on_death(dolphin_death)
}

fn dolphin_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn spawn_dolphin(app: &mut App) {
    let going_right = rand::thread_rng().gen_bool(0.5);
    let builder = build_dolphin(app.width, app.height, going_right);
    app.spawn(builder);
}

// ---------------------------------------------------------------------------
// Submarine — yellow submarine cruising underwater
// ---------------------------------------------------------------------------

const SUB_ART: &str = "\
         __|__\n\
         |   |\n\
  ___   _|___|_   ___\n\
 | + |-/ .   . \\-| + |\n\
 |___|{  (o) (o) }|___|\n\
       \\  ~~~  /\n\
  ~~~~~~`-----'~~~~~~";

const SUB_MASK: &str = "\
         WWWWW\n\
         W   W\n\
  YYY   YYYYYYY   YYY\n\
 YYYYY YYYYYYYYY YYYYY\n\
 YYYYYYYY WW  WW YYYYYY\n\
       Y  CCC  Y\n\
  CCCCCCYYYYYYYCCCCCC";

fn build_submarine(screen_w: u16, screen_h: u16, going_right: bool) -> EntityBuilder {
    let mut rng = rand::thread_rng();

    // Submarine is symmetric — same art for both directions
    let (art, mask_str) = (SUB_ART, SUB_MASK);

    let colored_mask = color::rand_color(mask_str);
    let shape = Shape::single(art, Some(&colored_mask), Some(' '));

    let frame = &shape.frames[0];
    let speed = rng.gen_range(0.8..1.5_f64);
    let vx = if going_right { speed } else { -speed };
    let x = if going_right {
        -(frame.width as f64)
    } else {
        screen_w as f64
    };

    // Submarine cruises mid-water
    let min_y = 8.0;
    let max_y = (screen_h as f64) - (frame.height as f64) - 4.0;
    let y = rng.gen_range(min_y..max_y.max(min_y + 1.0));

    EntityBuilder::new(EntityType::Submarine, "submarine", shape)
        .position(x, y, 15)
        .velocity(vx, 0.0)
        .color(Color::BrightYellow)
        .die_offscreen()
        .on_death(submarine_death)
}

fn submarine_death(ctx: &DeathContext) -> Vec<EntityCommand> {
    let builder = build_random(ctx.screen_width, ctx.screen_height, ctx.classic_mode);
    vec![EntityCommand::Spawn(builder)]
}

fn spawn_submarine(app: &mut App) {
    let going_right = rand::thread_rng().gen_bool(0.5);
    let builder = build_submarine(app.width, app.height, going_right);
    app.spawn(builder);
}

// ---------------------------------------------------------------------------
// Fishhook
// ---------------------------------------------------------------------------

fn spawn_hook(app: &mut App) {
    let mut rng = rand::thread_rng();
    // Random x position, clamped away from edges and castle zone
    let castle_left = app.width.saturating_sub(60);
    let x = rng.gen_range(10..castle_left.max(11)) as f64;

    let shape = build_hook_shape();

    // Start above the screen — the line is LINE_ROWS tall so the hook
    // begins way offscreen and the line feeds down through the waterline
    let start_y = -(HOOK_HEIGHT as f64) + 2.0;
    let builder = EntityBuilder::new(EntityType::Hook, "hook", shape)
        .position(x, start_y, 6) // z=6: same depth as waterline
        .velocity(0.0, HOOK_SPEED)
        .color(Color::White)
        .on_update(hook_update)
        .on_death(hook_death);

    app.spawn(builder);
}
