use crate::{EntityIndex, State, math_utils};

use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{max, min};
use std::f32::consts::SQRT_2;
use std::sync::Arc;

use hecs::World;
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

#[derive(Copy, Clone)]
pub enum DirectionalInputTypes {
    Attack
}

#[derive(Debug, Clone, Copy)]
pub struct Player;

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

// pub trait Entity {
//     fn get_x(&self) -> i32;
//     fn get_y(&self) -> i32;
//     fn get_display(&self) -> Display;
//
//     fn set_x(&mut self, x: i32);
//     fn set_y(&mut self, y: i32);
//
//     fn set_display(&mut self, display: Display);
//
//     fn move_by(&mut self, _dx: i32, _dy: i32);
// }

pub struct SelfDestructAI {
    pub turns_left: i32,
}

impl SelfDestructAI {

    pub fn on_turn(state: &mut State, me: EntityIndex) {
        let mut sd_ai = state.ecs.get_mut::<SelfDestructAI>(me).unwrap();
        sd_ai.turns_left -= 1;
        if sd_ai.turns_left < 0 {
            drop(sd_ai);
            state.ecs.despawn(me).expect("Failed to despawn an entity");
        }
    }

}

pub struct BasicEntity {
    pub x: i32,
    pub y: i32,
    pub d: Display,
}

//Entities with stat blocks and complex interactions etc
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatBlock {
    pub def: EntityStat,
    pub atk: EntityStat,
    pub hp: EntityStat,

    pub dead: bool,
}

impl Default for StatBlock {
    fn default() -> Self {
        StatBlock {
            atk: EntityStat::new("Attack", 0),
            def: EntityStat::new("Defense", 0),
            hp: EntityStat::new("Hit Points", 0),
            dead: false,
        }
    }
}

impl StatBlock {

    pub fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder
            .append(format!("HP: {}", self.hp.get_total()).as_str())
            .ln()
            .append(format!("ATK: {}", self.atk.get_total()).as_str())
            .ln()
            .append(format!("DEF: {}", self.def.get_total()).as_str());
    }

    pub fn take_damage(&mut self, damage: i32) -> bool {
        self.hp.decrement(max(0, damage - self.def.get_total()) );

        if self.hp.get_total() <= 0 && !self.dead {
            self.dead = true;
            return true;
        }

        return false;
    }
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

#[derive(Clone, Copy)]
pub struct ZombieAI;

impl ZombieAI {

    pub fn on_turn(state: &mut State, me: EntityIndex) {
        //let be = ecs.get::<BasicEntity>(e).unwrap();
        let plr_pos = state.get_player();
        let be_comp = state.ecs.get_mut::<BasicEntity>(me).unwrap();

        let dx = plr_pos.get_x() - be_comp.get_x();
        let dy = plr_pos.get_y() - be_comp.get_y();

        let dist = ((dx * dx + dy * dy) as f32).sqrt();

        let dx = dx / max(1, dx.abs());
        let dy = dy / max(1, dy.abs());

        let me_stats = state.ecs.get_mut::<StatBlock>(me).unwrap();

        if me_stats.dead { return; }

        if dist - 0.01_f32 < SQRT_2 {
            // Attack here
            let mut plr_stats = state.get_player_stat_block();
            plr_stats.take_damage(me_stats.atk.get_total());
        } else if dist < 5.0 && math_utils::chance(0.9) {
            // Move here
            drop(be_comp);
            drop(plr_pos);
            drop(me_stats);
            state.move_entity_by(me, dx, dy);
        }
    }

}

pub struct PlayerAI;

impl PlayerAI {
    pub fn on_turn(state: &mut State, e: EntityIndex) {
        let st_bl = &mut *state.ecs.get_mut::<StatBlock>(e).unwrap();
        st_bl.hp.increment(1);
    }
}

impl BasicEntity {

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_display(&self) -> Display {
        self.d
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn set_display(&mut self, display: Display) {
        self.d = display;
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
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
    pub art: Arc<XpFile>,
}

impl EntityView {
    pub fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder.centered(self.name.as_str()).ln();
    }
}

pub trait EntityLootHandler {
    fn handle_loot(&mut self, state: &crate::State);
}

pub struct SpiderLoot { pub id: EntityIndex, pub max_atk: i32}

impl EntityLootHandler for SpiderLoot {
    fn handle_loot(&mut self, state: &crate::State) {
        // if math_utils::chance(0.3) {
        //     //Improve player's atk up to 5
        //     let mut player_stats = state
        //         .stat_blocks[0]
        //         .unwrap_ref_mut();
        //
        //     let hp_cur_max = player_stats.hp.get_max();
        //     if hp_cur_max < 30 {
        //         player_stats.hp.set_max(
        //             hp_cur_max + 1
        //         );
        //     }
        //
        //     let atk_cur_max = player_stats.atk.get_max();
        //     if atk_cur_max < self.max_atk {
        //         player_stats.atk.set(
        //             atk_cur_max + 1
        //         );
        //     }
        // }
    }
}
