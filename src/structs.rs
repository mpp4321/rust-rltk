use crate::{EntityIndex, State, math_utils};

use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{max, min};
use std::rc::Rc;

use rltk::{RGB, TextBuilder, XpFile};
use serde::{Deserialize, Serialize};

pub mod map_utils {
    use std::{cell::Cell, fs::File, io::Read};
    use serde::{Deserialize, Serialize};

    use crate::MEEntity;

    use super::TileType;

    #[derive(Serialize, Deserialize)]
    pub struct MapDescriptor {
        pub width: i32,
        pub height: i32,
        pub tiles: Vec<TileType>,
        pub entities: Vec<Option<MEEntity>>,
    }

    pub fn load_from_file(file_name: &'static str) -> MapDescriptor {
        let mut file = File::open(file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let map_tiles: MapDescriptor = serde_json::from_str(&contents).unwrap();
        map_tiles
    }

    pub fn map_to_cells(tiles: Vec<TileType>) -> Vec<Cell<TileType>> {
        tiles.into_iter().map(|tile| Cell::new(tile)).collect()
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct EntityStat {
    name: String,
    base: i32,
    bonus: i32,
    max: i32,
}

impl EntityStat {

    pub fn increment(&mut self, amount: i32) {
        self.base = max(-self.bonus, self.base + amount);
        self.base = min(self.max, self.base);
    }

    pub fn decrement(&mut self, amount: i32) {
        self.base = max(-self.bonus, self.base - amount);
    }

    pub fn get_total(&self) -> i32 {
        self.base + self.bonus
    }

    pub fn get_base(&self) -> i32 {
        self.base
    }

    pub fn get_max(&self) -> i32 {
        self.max
    }

    pub fn set_max(&mut self, max: i32) {
        self.max = max;
    }

    pub fn new(name: &'static str, start: i32) -> Self {
        EntityStat {
            name: String::from(name),
            base: start,
            bonus: 0,
            max: start,
        }
    }

    pub fn set(&mut self, arg: i32) {
        self.base = arg;
        self.max = arg;
    }

}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Wall(Display),
    Floor(Display),
    Portal(Display, usize, i32, i32)
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    pub glyph: u16,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
}

impl Display {
    pub fn get_fg(&self) -> RGB {
        return RGB::named(self.fg);
    }

    pub fn get_bg(&self) -> RGB {
        return RGB::named(self.bg);
    }
}

pub trait EntityAI {
    fn get_id(&self) -> EntityIndex;

    fn on_turn(&mut self, state: &State);
    fn on_remove(&mut self, state: &State);
    fn on_death(&mut self, state: &State);
}

//Just things that every entity has and needs for rendering
pub trait Entity {
    fn get_id(&self) -> EntityIndex;

    fn get_x(&self) -> i32;
    fn get_y(&self) -> i32;
    fn get_display(&self) -> Display;

    fn set_x(&mut self, x: i32);
    fn set_y(&mut self, y: i32);

    fn set_display(&mut self, display: Display);

    fn move_by(&mut self, _dx: i32, _dy: i32);
}

pub struct SelfDestructAI {
    id: EntityIndex,
    turns_left: i32,
}

impl EntityAI for SelfDestructAI {
    fn get_id(&self) -> EntityIndex {
        return self.id;
    }

    fn on_turn(&mut self, state: &State) {
        self.turns_left -= 1;
        if self.turns_left < 0 {
            state.queue_destruction(self.get_id());
        }
    }

    fn on_remove(&mut self, _state: &State) {}

    fn on_death(&mut self, _state: &State) {}
}

pub struct BasicEntity {
    pub id: EntityIndex,

    pub x: i32,
    pub y: i32,
    pub d: Display,
}

//Entities with stat blocks and complex interactions etc
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatBlock {
    pub id: EntityIndex,

    pub def: EntityStat,
    pub atk: EntityStat,
    pub hp: EntityStat,

    pub dead: bool,
}

impl Default for StatBlock {
    fn default() -> Self {
        StatBlock {
            id: 0,
            atk: EntityStat::new("Attack", 0),
            def: EntityStat::new("Defense", 0),
            hp: EntityStat::new("Hit Points", 0),
            dead: false,
        }
    }
}

impl StatBlock {
    pub fn get_id(&self) -> EntityIndex {
        return self.id;
    }

    pub fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder
            .append(format!("HP: {}", self.hp.get_total()).as_str())
            .ln()
            .append(format!("ATK: {}", self.atk.get_total()).as_str())
            .ln()
            .append(format!("DEF: {}", self.def.get_total()).as_str());
    }

    pub fn take_damage(&mut self, state: &State, damage: i32) -> bool {
        self.hp.decrement(max(0, damage - self.def.get_total()) );

        if self.hp.get_total() <= 0 && !self.dead {
            self.dead = true;
            state.ais.borrow_mut()[self.id] = Some(Box::new(SelfDestructAI { id: self.get_id(), turns_left: 5 }));
            state.get_entity(self.id).borrow_mut().set_display(Display {
                glyph: rltk::to_cp437('%'),
                fg: rltk::RED,
                bg: rltk::BLACK
            });

            if state.entity_loots[self.id].is_some() {
                let mut e_loot = state.entity_loots[self.id].as_ref().unwrap().borrow_mut();
                e_loot.handle_loot(state);
            }

            return true;
        }

        return false;
    }
}

pub struct ZombieAI {
    pub id: EntityIndex,
}

pub struct PlayerAI;

impl EntityAI for PlayerAI {
    fn get_id(&self) -> EntityIndex {
        return 0;
    }

    fn on_turn(&mut self, state: &State) {
        if math_utils::chance(0.3) {
            // Get the player's stat block
            let mut player_stats = state.stat_blocks[0].unwrap_ref_mut();
            player_stats.hp.increment(1);
        }
    }

    fn on_remove(&mut self, _: &State) {}
    fn on_death(&mut self, _: &State) {}
}

pub trait MyOptionTimeSaver<T> {
    fn unwrap_ref(&self) -> Ref<T>;
    fn unwrap_ref_mut(&self) -> RefMut<T>;
}

impl<T> MyOptionTimeSaver<T> for Option<RefCell<T>> {

    fn unwrap_ref(&self) -> Ref<T> {
        self.as_ref().unwrap().borrow()
    }

    fn unwrap_ref_mut(&self) -> RefMut<T> {
        self.as_ref().unwrap().borrow_mut()
    }

}

impl EntityAI for ZombieAI {
    fn get_id(&self) -> EntityIndex {
        return self.id;
    }

    fn on_turn(&mut self, state: &State) {
        let me = self.get_id();
        if math_utils::chance(0.5) {
            return;
        }

        let player_pos = (
            state.get_entity(0).borrow().get_x(),
            state.get_entity(0).borrow().get_y(),
        );
        let mut me = state.get_entity(me).borrow_mut();
        let zombie_pos = (me.get_x(), me.get_y());
        //Calculate the direction to the player from zombie_pos
        let dx = player_pos.0 - zombie_pos.0;
        let dy = player_pos.1 - zombie_pos.1;
        //Calculate the distance as float
        let distance = (((dx * dx) + (dy * dy)) as f32).sqrt();
        if distance > 3.0 {
            return;
        }
        //Normalize dx, dy

        let dx = dx / max(1, dx.abs());
        let dy = dy / max(1, dy.abs());

        const SQRT_2DIST: f32 = 0.01 + std::f64::consts::SQRT_2 as f32;
        if distance < SQRT_2DIST {
            state.stat_blocks[0]
                .unwrap_ref_mut()
                .take_damage(state, state.stat_blocks[me.get_id()].unwrap_ref().atk.get_total());
        } else if state.can_move(me.get_x() + dx, me.get_y() + dy) {
            me.move_by(dx, dy);
        }
    }
    fn on_remove(&mut self, _: &State) {}
    fn on_death(&mut self, _: &State) {}
}

impl Entity for BasicEntity {
    fn get_id(&self) -> EntityIndex {
        return self.id;
    }

    fn get_x(&self) -> i32 {
        self.x
    }

    fn get_y(&self) -> i32 {
        self.y
    }

    fn get_display(&self) -> Display {
        self.d
    }

    fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    fn set_display(&mut self, display: Display) {
        self.d = display;
    }

    fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }
}

pub struct Camera {
    x: i32,
    y: i32,

    pub x_offset: i32,
    pub y_offset: i32,

    pub dx: i32,
    pub dy: i32,

    pub time_till_tween: f32,
}

impl Camera {
    pub fn update_xy(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;

        self.dx = 0;
        self.dy = 0;
    }

    pub fn new(xoff: i32, yoff: i32) -> Camera {
        Camera {
            x: 0,
            y: 0,
            x_offset: xoff,
            y_offset: yoff,
            dx: 0,
            dy: 0,
            time_till_tween: 0.2,
        }
    }

    pub fn mod_x(&self) -> i32 {
        return self.x + self.x_offset;
    }

    pub fn mod_y(&self) -> i32 {
        return self.y + self.y_offset;
    }

    pub fn get_x(&self) -> i32 {
        return self.x;
    }

    pub fn get_y(&self) -> i32 {
        return self.y;
    }

    pub fn tween_tick(&mut self, dt_ms: f32) {
        self.time_till_tween -= dt_ms / 1000.0;
        if self.time_till_tween <= 0.0 {
            self.time_till_tween = 0.2;

            //Move the camera's x, y towards the dx, dy by 1 unit
            let dx_unit = self.dx / max(1, self.dx.abs());
            let dy_unit = self.dy / max(1, self.dy.abs());

            if dx_unit == 0 && dy_unit == 0 {
                self.time_till_tween = 0.5;
            }

            self.dx -= dx_unit;
            self.dy -= dy_unit;

            self.x += dx_unit;
            self.y += dy_unit;
        }
    }
}

impl Camera {

    //Transforms a world point into a screen point
    pub fn transform_point(&self, point: (i32, i32)) -> (i32, i32) {
        (point.0 - self.x_offset - self.x, point.1 - self.y_offset - self.y)
    }

    // fn untransform_point(&self, point: (i32, i32)) -> (i32, i32) {
    //     (point.0 + self.x, point.1 + self.y)
    // }
}

pub struct EntityView {
    pub name: String,
    pub art: Rc<XpFile>,
}

impl EntityView {
    pub fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder.centered(self.name.as_str()).ln();
    }
}
pub trait EntityLootHandler {
    fn get_id(&self) -> EntityIndex;
    fn handle_loot(&mut self, state: &crate::State);
}

pub struct SpiderLoot { pub id: EntityIndex, pub max_atk: i32}

impl EntityLootHandler for SpiderLoot {
    fn get_id(&self) -> EntityIndex {
        self.id
    }

    fn handle_loot(&mut self, state: &crate::State) {
        if math_utils::chance(0.3) {
            //Improve player's atk up to 5
            let mut player_stats = state
                .stat_blocks[0]
                .unwrap_ref_mut();

            let hp_cur_max = player_stats.hp.get_max();
            if hp_cur_max < 30 {
                player_stats.hp.set_max(
                    hp_cur_max + 1
                );
            }

            let atk_cur_max = player_stats.atk.get_max();
            if atk_cur_max < self.max_atk {
                player_stats.atk.set(
                    atk_cur_max + 1
                );
            }
        }
    }
}