use std::collections::HashMap;

use rand::Rng;

use crate::art;
use crate::entity::{
    DeathContext, Entity, EntityBuilder, EntityCommand, EntityId, EntityType, OffscreenBehavior,
    UpdateContext,
};
use crate::renderer::FrameBuffer;

pub struct App {
    entities: HashMap<EntityId, Entity>,
    next_id: EntityId,
    pub width: u16,
    pub height: u16,
    pub paused: bool,
    pub classic_mode: bool,
    /// Custom image shapes that respawn when they go offscreen.
    custom_shapes: Vec<crate::shape::Shape>,
}

impl App {
    pub fn new(width: u16, height: u16, classic: bool) -> Self {
        App {
            entities: HashMap::new(),
            next_id: 1,
            width,
            height,
            paused: false,
            classic_mode: classic,
            custom_shapes: Vec::new(),
        }
    }

    pub fn init_scene(&mut self) {
        self.entities.clear();

        // Water lines
        art::environment::spawn_water(self);
        // Castle
        art::environment::spawn_castle(self);
        // Seaweed: scale count with diminishing returns for large terminals
        let seaweed_count = if self.width < 60 {
            // Small terminals: at least 2
            (self.width / 20).max(2) as usize
        } else if self.width > 200 {
            // Large terminals: cap growth with sqrt-based scaling
            let base = 200u16 / 15;
            let extra = ((self.width - 200) as f64).sqrt() as u16;
            (base + extra) as usize
        } else {
            (self.width / 15) as usize
        };
        for _ in 0..seaweed_count {
            art::environment::spawn_seaweed(self);
        }
        // Fish: scale count with minimum and maximum density
        let screen_area = (self.height.saturating_sub(9) as u32) * (self.width as u32);
        let fish_count = if self.width < 80 {
            // Small terminals: ensure at least 3 fish
            (screen_area / 350).max(3) as usize
        } else if self.width > 200 {
            // Large terminals: cap density to prevent overcrowding.
            // Use a larger divisor beyond 200 cols to flatten the curve.
            let base_area = (self.height.saturating_sub(9) as u32) * 200;
            let extra_area = screen_area.saturating_sub(base_area);
            let base_count = base_area / 350;
            let extra_count = extra_area / 600;
            (base_count + extra_count) as usize
        } else {
            (screen_area / 350) as usize
        };
        for _ in 0..fish_count {
            art::fish::spawn_fish_scattered(self, self.classic_mode);
        }
        // A few ducks scattered across the water — keep it subtle
        let duck_count = rand::thread_rng().gen_range(2..=3_usize);
        for _ in 0..duck_count {
            art::duck::spawn_duck_scattered(self);
        }
        // One random object
        art::random::spawn_random(self, self.classic_mode);
        // Custom image entities
        self.spawn_custom_images();
    }

    pub fn spawn(&mut self, builder: EntityBuilder) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let entity = builder.build(id);
        self.entities.insert(id, entity);
        id
    }

    pub fn update(&mut self, dt: f64) {
        let ctx = UpdateContext {
            screen_width: self.width,
            screen_height: self.height,
            dt,
        };

        // Collect update commands
        let mut commands: Vec<(EntityId, Vec<EntityCommand>)> = Vec::new();
        let ids: Vec<EntityId> = self.entities.keys().copied().collect();

        for id in &ids {
            if let Some(entity) = self.entities.get_mut(id) {
                // Move entity
                entity.x += entity.vx * dt;
                entity.y += entity.vy * dt;

                // Advance animation frames
                if entity.animation_speed > 0.0 && entity.shape.frames.len() > 1 {
                    entity.frame_timer += dt;
                    if entity.frame_timer >= entity.animation_speed {
                        entity.frame_timer -= entity.animation_speed;
                        entity.current_frame =
                            (entity.current_frame + 1) % entity.shape.frames.len();
                    }
                }

                // Run update callback
                if let Some(on_update) = entity.on_update {
                    let cmds = on_update(entity, &ctx);
                    if !cmds.is_empty() {
                        commands.push((*id, cmds));
                    }
                }
            }
        }

        // Check offscreen
        let mut to_kill: Vec<EntityId> = Vec::new();
        for id in &ids {
            if let Some(entity) = self.entities.get(id) {
                if let OffscreenBehavior::Die = entity.offscreen {
                    if entity.is_offscreen(self.width, self.height) {
                        to_kill.push(*id);
                    }
                }
            }
        }

        // Process commands
        for (entity_id, cmds) in commands {
            for cmd in cmds {
                match cmd {
                    EntityCommand::Spawn(builder) => {
                        self.spawn(builder);
                    }
                    EntityCommand::KillSelf => {
                        to_kill.push(entity_id);
                    }
                }
            }
        }

        // Kill entities and run death callbacks
        let classic = self.classic_mode;
        let w = self.width;
        let h = self.height;
        let mut spawns: Vec<EntityBuilder> = Vec::new();

        for id in to_kill {
            if let Some(entity) = self.entities.remove(&id) {
                if let Some(on_death) = entity.on_death {
                    let death_ctx = DeathContext {
                        entity_type: entity.entity_type.clone(),
                        screen_width: w,
                        screen_height: h,
                        classic_mode: classic,
                    };
                    let cmds = on_death(&death_ctx);
                    for cmd in cmds {
                        match cmd {
                            EntityCommand::Spawn(builder) => {
                                spawns.push(builder);
                            }
                            EntityCommand::KillSelf => {} // already dead
                        }
                    }
                }
            }
        }

        for builder in spawns {
            self.spawn(builder);
        }

        // Respawn custom images that went offscreen
        if !self.custom_shapes.is_empty() {
            self.respawn_custom_images();
        }
    }

    pub fn render(&self) -> FrameBuffer {
        let mut fb = FrameBuffer::new(self.width, self.height);

        // Sort entities by Z descending (highest Z = furthest back = rendered first)
        let mut sorted: Vec<&Entity> = self.entities.values().collect();
        sorted.sort_by(|a, b| b.z.cmp(&a.z));

        for entity in sorted {
            let frame = entity.current_frame();
            let ex = entity.x as i32;
            let ey = entity.y as i32;

            for (row, (cell_row, color_row)) in
                frame.cells.iter().zip(frame.colors.iter()).enumerate()
            {
                for (col, (cell, color)) in
                    cell_row.iter().zip(color_row.iter()).enumerate()
                {
                    if let Some(ch) = cell {
                        let screen_x = ex + col as i32;
                        let screen_y = ey + row as i32;
                        let c = color.unwrap_or(entity.default_color);
                        fb.set(screen_x, screen_y, *ch, c);
                    }
                }
            }
        }

        fb
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
    }

    pub fn clear_all(&mut self) {
        self.entities.clear();
    }

    /// Check for collisions between physical entities (fish vs teeth, bubble vs waterline)
    pub fn check_collisions(&mut self) {
        let physical: Vec<(EntityId, EntityType, i32, i32, u16, u16, i32)> = self
            .entities
            .values()
            .filter(|e| e.physical)
            .map(|e| {
                let f = e.current_frame();
                (
                    e.id,
                    e.entity_type.clone(),
                    e.x as i32,
                    e.y as i32,
                    f.width,
                    f.height,
                    e.z,
                )
            })
            .collect();

        let mut kills: Vec<EntityId> = Vec::new();
        let mut splats: Vec<(i32, i32, i32)> = Vec::new();

        for i in 0..physical.len() {
            for j in (i + 1)..physical.len() {
                let a = &physical[i];
                let b = &physical[j];

                // Simple AABB overlap
                let ax2 = a.2 + a.4 as i32;
                let ay2 = a.3 + a.5 as i32;
                let bx2 = b.2 + b.4 as i32;
                let by2 = b.3 + b.5 as i32;

                if a.2 < bx2 && ax2 > b.2 && a.3 < by2 && ay2 > b.3 {
                    // Collision detected
                    match (&a.1, &b.1) {
                        (EntityType::Fish, EntityType::Teeth) => {
                            // Check fish height <= 5
                            if a.5 <= 5 {
                                kills.push(a.0);
                                splats.push((b.2, b.3, b.6));
                            }
                        }
                        (EntityType::Teeth, EntityType::Fish) => {
                            if b.5 <= 5 {
                                kills.push(b.0);
                                splats.push((a.2, a.3, a.6));
                            }
                        }
                        (EntityType::Bubble, EntityType::Waterline) => {
                            kills.push(a.0);
                        }
                        (EntityType::Waterline, EntityType::Bubble) => {
                            kills.push(b.0);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Process collision deaths
        let classic = self.classic_mode;
        let w = self.width;
        let h = self.height;
        let mut spawns: Vec<EntityBuilder> = Vec::new();

        for (sx, sy, sz) in splats {
            spawns.push(art::fish::splat_builder(sx, sy, sz));
        }

        for id in kills {
            if let Some(entity) = self.entities.remove(&id) {
                if let Some(on_death) = entity.on_death {
                    let death_ctx = DeathContext {
                        entity_type: entity.entity_type.clone(),
                        screen_width: w,
                        screen_height: h,
                        classic_mode: classic,
                    };
                    let cmds = on_death(&death_ctx);
                    for cmd in cmds {
                        if let EntityCommand::Spawn(builder) = cmd {
                            spawns.push(builder);
                        }
                    }
                }
            }
        }

        for builder in spawns {
            self.spawn(builder);
        }
    }

    /// Register a custom image shape for the aquarium. It will be spawned
    /// and respawned automatically.
    pub fn add_custom_shape(&mut self, shape: crate::shape::Shape) {
        self.custom_shapes.push(shape);
    }

    /// Spawn all registered custom image entities.
    pub fn spawn_custom_images(&mut self) {
        let shapes: Vec<crate::shape::Shape> = self.custom_shapes.clone();
        for shape in &shapes {
            self.spawn_one_custom_image(shape.clone());
        }
    }

    /// Spawn a single custom image entity that swims across the screen.
    fn spawn_one_custom_image(&mut self, shape: crate::shape::Shape) {
        use crate::color::Color;

        let mut rng = rand::thread_rng();
        let dir = rng.gen_range(0..2);
        let speed = rng.gen_range(0.5..1.5_f64);
        let frame = &shape.frames[0];

        // Start just offscreen so it appears quickly
        let x = if dir == 0 {
            -(frame.width as f64) * 0.5
        } else {
            self.width as f64 - (frame.width as f64) * 0.5
        };
        let vx = if dir == 0 { speed } else { -speed };

        // Place in the water area (below waterline at row 4, above bottom)
        let top_y = 6_i32;
        let bottom_y = self.height.saturating_sub(frame.height + 2) as i32;
        let y = if bottom_y > top_y {
            rng.gen_range(top_y..bottom_y) as f64
        } else {
            top_y as f64
        };

        let z = rng.gen_range(3..15);

        let builder = EntityBuilder::new(EntityType::Fish, "custom_image", shape)
            .position(x, y, z)
            .velocity(vx, 0.0)
            .color(Color::White)
            .die_offscreen();

        self.spawn(builder);
    }

    /// Respawn any custom images that have died (gone offscreen).
    /// Called during update to maintain custom image count.
    fn respawn_custom_images(&mut self) {
        let current_custom = self
            .entities
            .values()
            .filter(|e| e.name == "custom_image")
            .count();
        let target = self.custom_shapes.len();

        if current_custom < target {
            let shapes: Vec<crate::shape::Shape> = self.custom_shapes.clone();
            for _ in current_custom..target {
                // Pick a random custom shape to respawn
                let idx = rand::thread_rng().gen_range(0..shapes.len());
                self.spawn_one_custom_image(shapes[idx].clone());
            }
        }
    }

    /// Kill teeth entities when their shark dies
    pub fn kill_entities_of_type(&mut self, entity_type: &EntityType) {
        let to_remove: Vec<EntityId> = self
            .entities
            .values()
            .filter(|e| &e.entity_type == entity_type)
            .map(|e| e.id)
            .collect();
        for id in to_remove {
            self.entities.remove(&id);
        }
    }
}
