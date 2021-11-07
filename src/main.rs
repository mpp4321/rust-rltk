extern crate rltk;

mod entities;

use entities::entity_create;

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::cmp::max;
use std::fs::File;
use std::rc::Rc;

use rltk::{
    BResult, BTermBuilder, ColorPair, DrawBatch, GameState, Point, Rltk, TextBlock, TextBuilder,
    VirtualKeyCode, XpFile, RGB,
};

type EntityIndex = usize;

mod math_utils {
    use rand::Rng;

    pub fn clamp(value: i32, min: i32, max: i32) -> i32 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    pub fn chance(f: f32) -> bool {
        return rand::thread_rng().gen::<f32>() < f;
    }

    pub fn random_point(x1: i32, x2: i32, y1: i32, y2: i32) -> (i32, i32) {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(x1..x2);
        let y = rng.gen_range(y1..y2);
        return (x, y);
    }
}

#[derive(Copy, Clone, PartialEq)]
enum TileType {
    Wall,
    Floor,
    Object(Display),
}

#[derive(Copy, Clone, PartialEq)]
pub struct Display {
    glyph: u16,
    fg: RGB,
    bg: RGB,
}

pub trait EntityAI {
    fn get_id(&self) -> EntityIndex;

    fn on_turn(&mut self, me: EntityIndex, state: &State);
    fn on_remove(&mut self, me: EntityIndex, state: &State);
    fn on_death(&mut self, me: EntityIndex, state: &State);
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

struct SelfDestructAI {
    turns_left: i32,
}
impl EntityAI for SelfDestructAI {
    fn get_id(&self) -> EntityIndex {
        return 0;
    }

    fn on_turn(&mut self, me: EntityIndex, state: &State) {
        self.turns_left -= 1;
        if self.turns_left < 0 {
            state.queue_destruction(me);
        }
    }

    fn on_remove(&mut self, _me: EntityIndex, _state: &State) {}

    fn on_death(&mut self, _me: EntityIndex, _state: &State) {}
}

struct BasicEntity {
    id: EntityIndex,

    x: i32,
    y: i32,
    d: Display,
}

//Entities with stat blocks and complex interactions etc
#[derive(Copy, Clone, Debug)]
pub struct StatBlock {
    id: EntityIndex,

    def: i32,
    atk: i32,
    hp: i32,

    dead: bool,
}

impl Default for StatBlock {
    fn default() -> Self {
        StatBlock {
            id: 0,
            atk: 0,
            def: 0,
            hp: 0,
            dead: false,
        }
    }
}

impl StatBlock {
    fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder
            .append(format!("HP: {}", self.hp).as_str())
            .ln()
            .append(format!("ATK: {}", self.atk).as_str())
            .ln()
            .append(format!("DEF: {}", self.def).as_str());
    }

    fn take_damage(&mut self, state: &State, damage: i32) -> bool {
        self.hp -= damage;
        if self.hp <= 0 {
            self.hp = 0;
            self.dead = true;
            state.ais.borrow_mut()[self.id] = Some(Box::new(SelfDestructAI { turns_left: 5 }));
            state.get_entity(self.id).borrow_mut().set_display(Display {
                glyph: rltk::to_cp437('%'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            });

            return true;
        }

        return false;
    }
}

struct ZombieAI {
    id: EntityIndex,
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

    fn on_turn(&mut self, me: EntityIndex, state: &State) {
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
            state.stat_blocks[0].unwrap_ref_mut().take_damage(state, 1);
        } else if state.can_move(me.get_x() + dx, me.get_y() + dy) {
            me.move_by(dx, dy);
        }
    }
    fn on_remove(&mut self, _: EntityIndex, _: &State) {}
    fn on_death(&mut self, _: EntityIndex, _: &State) {}
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
}

impl Camera {
    fn transform_point(&self, point: (i32, i32)) -> (i32, i32) {
        (point.0 - self.x, point.1 - self.y)
    }

    fn untransform_point(&self, point: (i32, i32)) -> (i32, i32) {
        (point.0 + self.x, point.1 + self.y)
    }
}

pub struct EntityView {
    name: String,
    art: Rc<XpFile>,
}

impl EntityView {
    fn make_text_builder(&self, builder: &mut TextBuilder) {
        builder
            .centered(self.name.as_str())
            .ln();
    }
}

pub struct State {
    entities: Vec<Option<Box<RefCell<dyn Entity>>>>,
    ais: RefCell<Vec<Option<Box<dyn EntityAI>>>>,
    stat_blocks: Vec<Option<RefCell<StatBlock>>>,
    entity_views: Vec<Option<Rc<EntityView>>>,
    resources: Vec<Rc<XpFile>>,

    queued_destruction: RefCell<Vec<EntityIndex>>,

    free_slots: Vec<EntityIndex>,

    tiles: Vec<Cell<TileType>>,

    camera: RefCell<Camera>,

    waiting_for_directional_input: bool,
    directional_callback: Option<fn(&State, i32, i32)>,

    currently_viewed_art: Option<Rc<EntityView>>,
    currently_viewed_stat_block: Option<StatBlock>,
}

impl State {
    pub fn print_image_at(&self, x: i32, y: i32, entity_view: &EntityView, ctx: &mut Rltk) {
        ctx.render_xp_sprite(&entity_view.art, x, y);
    }

    pub fn queue_destruction(&self, slot: EntityIndex) {
        self.queued_destruction.borrow_mut().push(slot);
    }

    pub fn dispose_slot(&mut self, slot: EntityIndex) {
        self.entities[slot] = None;
        self.stat_blocks[slot] = None;
        self.ais.borrow_mut()[slot] = None;
        self.free_slots.push(slot);
    }

    pub fn consume_free_slot(&mut self) -> EntityIndex {
        if self.free_slots.len() > 0 {
            return self.free_slots.pop().unwrap();
        }
        return self.entities.len();
    }

    pub fn add_entity(
        &mut self,
        index: EntityIndex,
        entity: Option<Box<RefCell<dyn Entity>>>,
        ai: Option<Box<dyn EntityAI>>,
        stat_block: Option<RefCell<StatBlock>>,
        view: Option<Rc<EntityView>>,
    ) {
        self.entities.insert(index, entity);
        self.ais.borrow_mut().insert(index, ai);
        self.stat_blocks.insert(index, stat_block);
        self.entity_views.insert(index, view)
    }

    fn handle_directional_input(&mut self, key: VirtualKeyCode) -> bool {
        if !self.waiting_for_directional_input {
            return false;
        }
        if self.directional_callback.is_none() {
            self.waiting_for_directional_input = false;
            return false;
        }
        match key {
            VirtualKeyCode::Up => self.directional_callback.unwrap()(self, 0, -1),
            VirtualKeyCode::Down => self.directional_callback.unwrap()(self, 0, 1),
            VirtualKeyCode::Left => self.directional_callback.unwrap()(self, -1, 0),
            VirtualKeyCode::Right => self.directional_callback.unwrap()(self, 1, 0),

            VirtualKeyCode::Numpad8 => self.directional_callback.unwrap()(self, 0, -1),
            VirtualKeyCode::Numpad2 => self.directional_callback.unwrap()(self, 0, 1),
            VirtualKeyCode::Numpad4 => self.directional_callback.unwrap()(self, -1, 0),
            VirtualKeyCode::Numpad6 => self.directional_callback.unwrap()(self, 1, 0),

            VirtualKeyCode::Numpad7 => self.directional_callback.unwrap()(self, -1, -1),
            VirtualKeyCode::Numpad9 => self.directional_callback.unwrap()(self, 1, -1),
            VirtualKeyCode::Numpad1 => self.directional_callback.unwrap()(self, -1, 1),
            VirtualKeyCode::Numpad3 => self.directional_callback.unwrap()(self, 1, 1),
            _ => {}
        };
        return true;
    }

    fn handle_movement_input(&mut self, key: VirtualKeyCode, do_tick: &mut bool) {
        match key {
            VirtualKeyCode::Up => self.move_player_by(0, -1),
            VirtualKeyCode::Down => self.move_player_by(0, 1),
            VirtualKeyCode::Left => self.move_player_by(-1, 0),
            VirtualKeyCode::Right => self.move_player_by(1, 0),

            VirtualKeyCode::Numpad8 => self.move_player_by(0, -1),
            VirtualKeyCode::Numpad2 => self.move_player_by(0, 1),
            VirtualKeyCode::Numpad4 => self.move_player_by(-1, 0),
            VirtualKeyCode::Numpad6 => self.move_player_by(1, 0),

            VirtualKeyCode::Numpad7 => self.move_player_by(-1, -1),
            VirtualKeyCode::Numpad9 => self.move_player_by(1, -1),
            VirtualKeyCode::Numpad1 => self.move_player_by(-1, 1),
            VirtualKeyCode::Numpad3 => self.move_player_by(1, 1),

            VirtualKeyCode::Key1 => {
                self.waiting_for_directional_input = true;
                self.directional_callback = Some(|state, dx, dy| {
                    let (px, py) = (
                        state.get_entity(0).borrow().get_x() + dx,
                        state.get_entity(0).borrow().get_y() + dy,
                    );
                    for entity in state.entities.iter().flatten() {
                        let (ex, ey) = (entity.borrow().get_x(), entity.borrow().get_y());
                        if ex == px && ey == py {
                            //Callback here
                            let player_stats = state.stat_blocks[0].as_ref().unwrap().borrow();
                            let entity_id = entity.borrow().get_id();
                            match state.stat_blocks[entity_id].as_ref() {
                                Some(e) => {
                                    e.borrow_mut().take_damage(state, player_stats.atk);
                                }
                                None => {}
                            }
                        }
                    }
                });
                *do_tick = false;
            }
            _ => {}
        };
    }

    fn on_turn(&mut self) {
        for i in 0..self.entities.len() {
            let mut ai = std::mem::replace(&mut self.ais.borrow_mut()[i], None);

            match ai {
                Some(ref mut ai) => {
                    ai.on_turn(i, self);
                }
                None => {}
            }

            //Return the memory
            let _ = std::mem::replace(&mut self.ais.borrow_mut()[i], ai);
        }
    }

    fn generate_map() -> Vec<Cell<TileType>> {
        let mut map = vec![];
        for _ in 0..80 * 50 {
            if math_utils::chance(0.03) {
                map.push(Cell::new(TileType::Wall));
            } else {
                map.push(Cell::new(TileType::Floor));
            }
        }
        map
    }

    fn generate_entities(&mut self) {
        for _ in 0..80 * 50 {
            if !math_utils::chance(0.01) {
                continue;
            }
            let pos = math_utils::random_point(0, 80, 0, 50);

            if !math_utils::chance(0.3) {
                entity_create::create_goblin(self, pos);
            } else {
                entity_create::create_fire_elemental(self, pos);
            }
        }
    }

    fn new() -> State {
        let player = BasicEntity {
            id: 0,
            x: 1,
            y: 1,
            d: Display {
                glyph: '@' as u16,
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        };

        let f_c_dir = std::env::current_dir().unwrap();

        let mut state = State {
            tiles: State::generate_map(),
            entities: vec![],
            ais: RefCell::new(vec![]),
            stat_blocks: vec![],
            entity_views: vec![],
            resources: vec![
                Rc::new(
                    XpFile::read(&mut File::open(f_c_dir.join("dude.png.xp")).unwrap()).unwrap(),
                ),
                Rc::new(
                    XpFile::read(&mut File::open(f_c_dir.join("firelemental.xp")).unwrap()).unwrap(),
                ),
                Rc::new(
                    XpFile::read(&mut File::open(f_c_dir.join("guard.xp")).unwrap()).unwrap(),
                )
            ],
            queued_destruction: RefCell::new(vec![]),
            free_slots: vec![],

            camera: RefCell::new(Camera { x: -20, y: -20 }),

            waiting_for_directional_input: false,
            directional_callback: None,
            currently_viewed_art: None,
            currently_viewed_stat_block: None,
        };

        state.add_entity(
            0,
            Some(Box::new(RefCell::new(player))),
            None,
            Some(RefCell::new(StatBlock {
                id: 0,
                hp: 10,
                atk: 5,
                ..Default::default()
            })),
            None,
        );

        state.generate_entities();

        //Tiles is a flat square array make a wall around the edges of the square of TileType::Wall
        for x in 0..80 {
            state.set_tile(x, 0, TileType::Wall);
            state.set_tile(x, 49, TileType::Wall);
        }

        for y in 0..50 {
            state.set_tile(0, y, TileType::Wall);
            state.set_tile(79, y, TileType::Wall);
        }

        state
    }

    /* Assumes the entity is initialized will panic if not! */
    fn get_entity(&self, idx: usize) -> &Box<RefCell<dyn Entity>> {
        match self.entities[idx] {
            Some(ref entity) => entity,
            None => panic!("No entity at index {}", idx),
        }
    }

    fn set_tile(&mut self, x: i32, y: i32, t: TileType) {
        let idx = self.xy_idx(x, y);
        self.tiles[idx].set(t);
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * 80) + x as usize
    }

    fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % 80, idx as i32 / 80)
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < 80 && y >= 0 && y < 50
    }

    fn can_move(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && self.tiles[self.xy_idx(x, y)].get() == TileType::Floor
    }

    fn move_entity_by(&self, entity: EntityIndex, x: i32, y: i32) -> (i32, i32) {
        let mut entity_r = self.get_entity(entity).borrow_mut();
        let new_x = entity_r.get_x() + x;
        let new_y = entity_r.get_y() + y;
        if self.can_move(new_x, new_y) {
            entity_r.set_x(new_x);
            entity_r.set_y(new_y);
            return (x, y);
        }
        return (0, 0);
    }

    fn move_player_by(&self, x: i32, y: i32) {
        let deltas = self.move_entity_by(0, x, y);
        {
            let mut cm = self.camera.borrow_mut();
            cm.x += deltas.0;
            cm.y += deltas.1;
        }
    }

    fn draw_map(&self, g_db: &mut DrawBatch) {
        let l_x = math_utils::clamp(self.camera.borrow().x, 0, 80);
        let h_x = math_utils::clamp(self.camera.borrow().x + 40, 0, 80);
        let l_y = math_utils::clamp(self.camera.borrow().y, 0, 80);
        let h_y = math_utils::clamp(self.camera.borrow().y + 40, 0, 50);
        for x in l_x..h_x {
            for y in l_y..h_y {
                let tile = self.tiles[self.xy_idx(x, y)].get();
                let (x, y) = self.camera.borrow().transform_point((x, y));
                match tile {
                    TileType::Floor => {
                        g_db.set(
                            Point::new(x, y),
                            ColorPair::new(RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.)),
                            rltk::to_cp437('.'),
                        );
                    }
                    TileType::Wall => {
                        g_db.set(
                            Point::new(x, y),
                            ColorPair::new(RGB::from_f32(0., 1.0, 0.), RGB::from_f32(0., 0., 0.)),
                            rltk::to_cp437('#'),
                        );
                    }
                    TileType::Object(d) => {
                        g_db.set(Point::new(x, y), ColorPair::new(d.fg, d.bg), d.glyph);
                    }
                }
            }
        }

        for entity in self.entities.iter() {
            if entity.is_none() {
                continue;
            }
            let entity = entity.as_ref().unwrap().borrow();
            let (x, y) = (entity.get_x(), entity.get_y());
            if !(l_x..h_x).contains(&x) || !(l_y..h_y).contains(&y) {
                continue;
            }
            let (x, y) = self.camera.borrow().transform_point((x, y));
            g_db.set(
                Point::new(x, y),
                ColorPair::new(entity.get_display().fg, entity.get_display().bg),
                entity.get_display().glyph,
            );
        }

        //Draw directional arrows around player
        {
            let player = self.get_entity(0).borrow();
            let (x, y) = (player.get_x(), player.get_y());
            let (x, y) = self.camera.borrow().transform_point((x, y));

            const DRW_ARR: fn(i32, i32, char, &mut rltk::DrawBatch) =
                |x: i32, y: i32, c: char, ctx: &mut rltk::DrawBatch| {
                    ctx.set(
                        Point::new(x, y),
                        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
                        rltk::to_cp437(c),
                    );
                };

            if self.waiting_for_directional_input {
                //Draw the arrows using DRW_ARR around the player
                DRW_ARR(x + 1, y + 1, /*'↘'*/ '\\', g_db);
                DRW_ARR(x - 1, y + 1, /*'↙'*/ '/', g_db);
                DRW_ARR(x + 1, y - 1, /*'↗'*/ '/', g_db);
                DRW_ARR(x - 1, y - 1, /*'↖'*/ '\\', g_db);
                //The top left and right corners
                DRW_ARR(x, y - 1, '↑', g_db);
                DRW_ARR(x, y + 1, '↓', g_db);
                DRW_ARR(x + 1, y, '→', g_db);
                DRW_ARR(x - 1, y, '←', g_db);
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut g_db = DrawBatch::new();
        g_db.cls();

        // Draw a line down x = 41 of |
        for y in 0..50 {
            g_db.set(
                Point::new(40, y),
                ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
                rltk::to_cp437('|'),
            );
        }

        //Render Stat info
        {
            let mut stat_block_to_draw = *self.stat_blocks[0].unwrap_ref();
            if !self.currently_viewed_stat_block.is_none() {
                stat_block_to_draw = self.currently_viewed_stat_block.unwrap();
            }

            //Draw the stat block in a TextBlock

            let mut tb = TextBuilder::empty();

            if let Some(en_view) = self.entity_views[stat_block_to_draw.id].as_ref() {
                en_view.make_text_builder(&mut tb);
            } else {
                tb.centered("<UNNAMED>").ln();
            }

            stat_block_to_draw.make_text_builder(&mut tb);
            let mut tblock = TextBlock::new(41, 0, 40, 20);
            tblock
                .print(&tb)
                .expect("Too much text for stat block to render");
            tblock.render_to_draw_batch(&mut g_db);
        }

        if ctx.left_click {
            let (x, y) = ctx.mouse_pos();
            for entity in self.entities.iter() {
                if entity.is_none() {
                    continue;
                }
                let entity = entity.as_ref().unwrap().borrow();
                let (ex, ey) = (entity.get_x(), entity.get_y());
                let (ex, ey) = self.camera.borrow().transform_point((ex, ey));
                if ex == x && ey == y {
                    let view = self.entity_views[entity.get_id()].as_ref();
                    let stat_bl = self.stat_blocks[entity.get_id()].as_ref();
                    if let Some(view_inner) = view {
                        self.currently_viewed_art = Some(view_inner.clone());
                    }

                    if let Some(stat_block) = stat_bl {
                        self.currently_viewed_stat_block = Some(*stat_block.borrow());
                    }
                }
            }
        }

        //Handle keyboard input WASD movements using self.move_player
        match ctx.key {
            None => {}
            Some(key) => {
                let mut do_tick = true;

                if let Some(key) = ctx.key {
                    if key == VirtualKeyCode::Escape {
                        self.currently_viewed_art = None;
                        self.currently_viewed_stat_block = None;
                    }
                }

                if self.waiting_for_directional_input {
                    //Do the directional input callback here
                    self.handle_directional_input(key);
                    self.waiting_for_directional_input = false;
                } else {
                    self.handle_movement_input(key, &mut do_tick);
                }
                if do_tick {
                    self.on_turn();
                }
            }
        }

        self.draw_map(&mut g_db);

        g_db.submit(0).expect("Rendering error with draw batch");

        rltk::render_draw_buffer(ctx).expect("Rendering error");

        if let Some(ref c_view_art) = self.currently_viewed_art {
            self.print_image_at(41, 20, c_view_art, ctx)
        };

        //Clean up the queued disposed objects
        let qudesc = self.queued_destruction.replace(Vec::new());
        for indx in qudesc {
            self.dispose_slot(indx);
        }
    }
}

fn main() -> BResult<()> {
    let context = BTermBuilder::simple(80, 40).unwrap().build()?;
    let gs = State::new();
    rltk::main_loop(context, gs)
}
