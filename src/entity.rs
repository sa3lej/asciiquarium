use crate::color::Color;
use crate::shape::Shape;

pub type EntityId = u64;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntityType {
    Fish,
    Shark,
    Teeth,
    Bubble,
    Waterline,
    Seaweed,
    Castle,
    Ship,
    Whale,
    Monster,
    BigFish,
    Splat,
    Hook,
    RandomObject,
    Duck,
    Dolphin,
    Submarine,
}

#[derive(Clone, Debug)]
pub enum OffscreenBehavior {
    Keep,
    Die,
}

pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub name: String,
    pub shape: Shape,
    pub current_frame: usize,
    pub animation_speed: f64,
    pub frame_timer: f64,

    pub x: f64,
    pub y: f64,
    pub z: i32,
    pub vx: f64,
    pub vy: f64,

    pub default_color: Color,
    pub offscreen: OffscreenBehavior,
    pub physical: bool,

    pub on_update: Option<fn(&mut Entity, &UpdateContext) -> Vec<EntityCommand>>,
    pub on_death: Option<fn(&DeathContext) -> Vec<EntityCommand>>,

    pub extra: EntityExtra,
}

/// Extra data that varies by entity type.
#[derive(Clone, Debug, Default)]
pub struct EntityExtra {
    /// For teeth: the shark entity they follow.
    pub follow_entity: Option<EntityId>,
    /// For seaweed: the column it occupies.
    pub column: Option<u16>,
    /// For hook: whether the fishing rod has been spawned.
    pub rod_spawned: bool,
}

pub struct UpdateContext {
    pub screen_width: u16,
    pub screen_height: u16,
    pub dt: f64,
}

pub struct DeathContext {
    pub entity_type: EntityType,
    pub screen_width: u16,
    pub screen_height: u16,
    pub classic_mode: bool,
}

#[derive(Debug)]
pub enum EntityCommand {
    Spawn(EntityBuilder),
    KillSelf,
}

#[derive(Debug)]
pub struct EntityBuilder {
    pub entity_type: EntityType,
    pub name: String,
    pub shape: Shape,
    pub x: f64,
    pub y: f64,
    pub z: i32,
    pub vx: f64,
    pub vy: f64,
    pub default_color: Color,
    pub offscreen: OffscreenBehavior,
    pub physical: bool,
    pub animation_speed: f64,
    pub on_update: Option<fn(&mut Entity, &UpdateContext) -> Vec<EntityCommand>>,
    pub on_death: Option<fn(&DeathContext) -> Vec<EntityCommand>>,
    pub extra: EntityExtra,
}

impl EntityBuilder {
    pub fn new(entity_type: EntityType, name: &str, shape: Shape) -> Self {
        EntityBuilder {
            entity_type,
            name: name.to_string(),
            shape,
            x: 0.0,
            y: 0.0,
            z: 10,
            vx: 0.0,
            vy: 0.0,
            default_color: Color::White,
            offscreen: OffscreenBehavior::Keep,
            physical: false,
            animation_speed: 0.0,
            on_update: None,
            on_death: None,
            extra: EntityExtra::default(),
        }
    }

    pub fn position(mut self, x: f64, y: f64, z: i32) -> Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn velocity(mut self, vx: f64, vy: f64) -> Self {
        self.vx = vx;
        self.vy = vy;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.default_color = color;
        self
    }

    pub fn die_offscreen(mut self) -> Self {
        self.offscreen = OffscreenBehavior::Die;
        self
    }

    pub fn with_physics(mut self) -> Self {
        self.physical = true;
        self
    }

    pub fn on_update(
        mut self,
        f: fn(&mut Entity, &UpdateContext) -> Vec<EntityCommand>,
    ) -> Self {
        self.on_update = Some(f);
        self
    }

    pub fn on_death(mut self, f: fn(&DeathContext) -> Vec<EntityCommand>) -> Self {
        self.on_death = Some(f);
        self
    }

    pub fn animate(mut self, speed: f64) -> Self {
        self.animation_speed = speed;
        self
    }

    pub fn build(self, id: EntityId) -> Entity {
        Entity {
            id,
            entity_type: self.entity_type,
            name: self.name,
            shape: self.shape,
            current_frame: 0,
            animation_speed: self.animation_speed,
            frame_timer: 0.0,
            x: self.x,
            y: self.y,
            z: self.z,
            vx: self.vx,
            vy: self.vy,
            default_color: self.default_color,
            offscreen: self.offscreen,
            physical: self.physical,
            on_update: self.on_update,
            on_death: self.on_death,
            extra: self.extra,
        }
    }
}

impl Entity {
    pub fn current_frame(&self) -> &crate::shape::Frame {
        &self.shape.frames[self.current_frame]
    }

    pub fn is_offscreen(&self, screen_w: u16, screen_h: u16) -> bool {
        let frame = self.current_frame();
        let w = frame.width as f64;
        let h = frame.height as f64;

        self.x + w < 0.0
            || self.x >= screen_w as f64
            || self.y + h < 0.0
            || self.y >= screen_h as f64
    }
}
