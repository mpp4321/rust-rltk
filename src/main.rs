extern crate rltk;

mod entities;
mod structs;

use entities::{ entity_create };
use structs::*;

use serde::{Deserialize, Serialize};
use structs::map_utils::MapDescriptor;

use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::Path;
use std::rc::Rc;

use rltk::{
    BResult, BTermBuilder, ColorPair, DrawBatch, GameState, Point, Rltk, TextBlock, TextBuilder,
    VirtualKeyCode, XpFile, RGB,
};

pub type EntityIndex = usize;

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

//Describes an entity in the map editor
#[derive(Clone, Serialize, Deserialize)]
pub struct MEEntity {
    d: Display,
    name: String,
}

pub struct MapEditorState {
    width: i32,
    height: i32,

    map_tiles: Vec<TileType>,
    entities: Vec<Option<MEEntity>>,
    picked_tile: Display,
    picked_entity: Option<MEEntity>,
}

impl MapEditorState {
    fn new(width: i32, height: i32) -> Self {
        MapEditorState {
            width,
            height,
            map_tiles: vec![
                TileType::Floor(Display {
                    glyph: '.' as u16,
                    fg: rltk::WHITE,
                    bg: rltk::BLACK
                });
                (width * height) as usize
            ],
            entities: vec![None; (width * height) as usize],
            picked_entity: None,
            picked_tile: Display {
                glyph: '.' as u16,
                fg: rltk::WHITE,
                bg: rltk::BLACK,
            },
        }
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn in_bounds(&self, pos: (i32, i32)) -> bool {
        pos.0 >= 0 && pos.0 < self.width && pos.1 >= 0 && pos.1 < self.height
    }

    fn handle_l_click(&mut self, pos: (i32, i32)) {
        if !self.in_bounds(pos) {
            return;
        }
        let indx = self.xy_idx(pos.0, pos.1);
        self.map_tiles[indx] = TileType::Floor(self.picked_tile);
    }

    fn handle_r_click(&mut self, pos: (i32, i32)) {
        if !self.in_bounds(pos) {
            return;
        }
        let indx = self.xy_idx(pos.0, pos.1);
        self.map_tiles[indx] = TileType::Wall(self.picked_tile);
    }

    fn handle_e_click(&mut self, pos: (i32, i32)) {
        if !self.in_bounds(pos) {
            return;
        }
        let indx = self.xy_idx(pos.0, pos.1);
        self.entities[indx] = self.picked_entity.clone();
    }

    fn export_to_file(&self) {
        let mut file = File::create("output.map").unwrap();
        let s_str = serde_json::to_string(&MapDescriptor {
            tiles: self.map_tiles.clone(),
            width: self.width,
            height: self.height,
            entities: self.entities.clone(),
        })
        .unwrap();
        file.write(s_str.as_bytes()).unwrap();
    }

    fn draw_map(&self, ctx: &mut Rltk) {
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.xy_idx(i, j);

                match self.map_tiles[idx] {
                    TileType::Floor(ref t) => {
                        ctx.set(i, j, t.fg, t.bg, t.glyph);
                    }
                    TileType::Wall(ref t) => {
                        ctx.set(i, j, t.fg, t.bg, t.glyph);
                    }
                    TileType::Portal(ref t, _, _, _) => {
                        ctx.set(i, j, t.fg, t.bg, t.glyph);
                    }
                }

                match self.entities[idx] {
                    Some(ref e) => {
                        ctx.set(i, j, e.d.fg, e.d.bg, e.d.glyph);
                    }
                    None => {}
                }
            }
        }
    }

    fn get_input(&self) -> String {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        input.trim().to_string()
    }
}

impl GameState for MapEditorState {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.draw_map(ctx);

        let mouse_pos = ctx.mouse_pos();

        let a = &rltk::INPUT;
        let right_click = a.lock().is_mouse_button_pressed(1);
        if ctx.left_click {
            if ctx.shift {
                self.handle_e_click(mouse_pos);
            } else {
                self.handle_l_click(mouse_pos);
            }
        } else if right_click {
            self.handle_r_click(mouse_pos);
        }

        match ctx.key {
            Some(VirtualKeyCode::K) => {
                let _ = stdout().flush();
                let mut new_glyph = String::new();
                stdin().read_line(&mut new_glyph).expect("Invalid input");
                let n_glyph_char = new_glyph.trim().parse::<char>().unwrap() as u16;
                self.picked_tile.glyph = n_glyph_char;

                new_glyph = String::new();
                stdin().read_line(&mut new_glyph).expect("Invalid input");

                // Self::remove_trailing_new_line(&mut new_glyph);
                let fg = new_glyph
                    .split(",")
                    .map(|a| a.trim())
                    .map(|a| a.parse::<u8>().unwrap())
                    .collect::<Vec<u8>>();
                let fg = (fg[0], fg[1], fg[2]);

                new_glyph = String::new();
                stdin().read_line(&mut new_glyph).expect("Invalid input");

                let bg = new_glyph
                    .split(",")
                    .map(|a| a.trim())
                    .map(|a| a.parse::<u8>().unwrap())
                    .collect::<Vec<u8>>();
                let bg = (bg[0], bg[1], bg[2]);

                self.picked_tile.bg = bg;
                self.picked_tile.fg = fg;
            }
            Some(VirtualKeyCode::R) => {
                let idx = self.xy_idx(mouse_pos.0, mouse_pos.1);
                self.entities[idx] = None;
            }
            Some(VirtualKeyCode::E) => {
                let _ = stdout().flush();
                let entity_name = self.get_input();
                let entity_display = match entity_name.as_str() {
                    "Goblin" => Display {
                        glyph: 'g' as u16,
                        fg: rltk::BLACK,
                        bg: rltk::RED,
                    },
                    "SFElemental" => Display {
                        glyph: '*' as u16,
                        fg: rltk::BLACK,
                        bg: rltk::RED,
                    },
                    "Spider" => Display {
                        glyph: 's' as u16,
                        fg: rltk::BLACK,
                        bg: rltk::RED,
                    },
                    "KSpider" => Display {
                        glyph: 'S' as u16,
                        fg: rltk::BLACK,
                        bg: rltk::RED,
                    },
                    _ => panic!("Unknown entity!"),
                };
                self.picked_entity = Some(MEEntity {
                    name: entity_name,
                    d: entity_display,
                });
            }
            Some(VirtualKeyCode::S) => {
                self.export_to_file();
            }
            Some(VirtualKeyCode::L) => {
                let md = structs::map_utils::load_from_file("output.map");
                self.map_tiles = md.tiles;
                self.width = md.width;
                self.height = md.height;
                self.entities = md.entities;
            }
            Some(VirtualKeyCode::P) => {
                let pos = mouse_pos;
                if self.in_bounds(pos) {
                    let idx = self.xy_idx(pos.0, pos.1);
                    self.picked_tile = match self.map_tiles[idx] {
                        TileType::Floor(ref t) => t.clone(),
                        TileType::Wall(ref t) => t.clone(),
                        TileType::Portal(ref t, _, _, _) => t.clone()
                    };
                }
            }
            Some(VirtualKeyCode::F) => {
                //Fill map with current tile as floor
                for i in 0..self.width {
                    for j in 0..self.height {
                        let idx = self.xy_idx(i, j);
                        self.map_tiles[idx] = TileType::Floor(self.picked_tile.clone());
                    }
                }
            }
            Some(VirtualKeyCode::A) => {
                let _ = stdout().flush();
                let portal_dir = self.get_input().parse::<usize>().unwrap();
                let x = self.get_input().parse::<i32>().unwrap();
                let y = self.get_input().parse::<i32>().unwrap();
                let idx = self.xy_idx(mouse_pos.0, mouse_pos.1);
                self.map_tiles[idx] = TileType::Portal(self.picked_tile.clone(), portal_dir, x, y);
            }
            Some(_) => {}
            None => {}
        }
    }
}

pub struct State {
    entities: Vec<Option<Box<RefCell<dyn Entity>>>>,
    ais: RefCell<Vec<Option<Box<dyn EntityAI>>>>,
    stat_blocks: Vec<Option<RefCell<StatBlock>>>,
    entity_views: Vec<Option<Rc<EntityView>>>,
    resources: Vec<Rc<XpFile>>,
    entity_loots: Vec<Option<Box<RefCell<dyn EntityLootHandler>>>>,

    queued_destruction: RefCell<Vec<EntityIndex>>,

    free_slots: Vec<EntityIndex>,

    map_width: i32,
    map_height: i32,
    tiles: Vec<Cell<TileType>>,

    camera: RefCell<Camera>,

    waiting_for_directional_input: bool,
    directional_callback: Option<fn(&State, i32, i32)>,

    currently_viewed_art: Option<Rc<EntityView>>,
    currently_viewed_stat_block: Option<StatBlock>,

    until_player_save: f32,

    portal_locations: Vec<&'static str>,
    destination_next_tick: RefCell<Option<(usize, i32, i32)>>
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
        self.entity_views.insert(index, view);
        self.entity_loots.insert(index, None);
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
            VirtualKeyCode::S => self.save_player(),

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
                            let dmg = {
                                let player_stats = state.stat_blocks[0].as_ref().unwrap().borrow();
                                player_stats.atk.get_total()
                            };
                            let entity_id = entity.borrow().get_id();
                            match state.stat_blocks[entity_id].as_ref() {
                                Some(e) => {
                                    e.borrow_mut()
                                        .take_damage(state, dmg);
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
                    ai.on_turn(self);
                }
                None => {}
            }

            //Return the memory
            let _ = std::mem::replace(&mut self.ais.borrow_mut()[i], ai);
        }
    }

    fn generate_map(width: i32, height: i32) -> Vec<Cell<TileType>> {
        let mut map = vec![];
        for _ in 0..width * height {
            if math_utils::chance(0.03) {
                map.push(Cell::new(TileType::Wall(Display {
                    glyph: '^' as u16,
                    fg: rltk::GREEN,
                    bg: rltk::BLACK,
                })));
            } else {
                map.push(Cell::new(TileType::Floor(Display {
                    glyph: '.' as u16,
                    fg: rltk::WHITE,
                    bg: rltk::BLACK,
                })));
            }
        }
        map
    }

    fn generate_entities(&mut self) {
        for _ in 0..(self.map_width * self.map_height) {
            if !math_utils::chance(0.01) {
                continue;
            }

            let pos = math_utils::random_point(1, self.map_width - 1, 1, self.map_height - 1);

            if !math_utils::chance(0.3) {
                entity_create::create_goblin(self, pos);
            } else {
                entity_create::create_fire_elemental(self, pos);
            }
        }
    }

    fn load_entities_from_map(state: &mut State, entities: &Vec<Option<MEEntity>>) {
        for load_entity in entities.iter().enumerate() {
            if !load_entity.1.is_some() {
                continue;
            }
            let (x, y) = state.idx_xy(load_entity.0);
            let m_entity = load_entity.1.as_ref().unwrap();
            entity_create::resolve_entity_string(state, (x, y), m_entity.name.as_str());
        }

    }

    fn new() -> State {
        let player = BasicEntity {
            id: 0,
            x: 1,
            y: 1,
            d: Display {
                glyph: '@' as u16,
                fg: rltk::YELLOW,
                bg: rltk::BLACK,
            },
        };

        let f_c_dir = std::env::current_dir().unwrap();

        let rc_load = |s: &'static str| {
            Rc::new(
                XpFile::read(
                    &mut File::open(f_c_dir.join(s)).expect(
                        format!(
                            "Could not find the file: {}",
                            f_c_dir.join(s).as_path().to_str().unwrap()
                        )
                        .as_str(),
                    ),
                )
                .unwrap(),
            )
        };

        let load_map = map_utils::load_from_file("output.map");

        let mut state = State {
            map_width: load_map.width,
            map_height: load_map.height,
            // tiles: State::generate_map(32, 32),
            tiles: map_utils::map_to_cells(load_map.tiles),
            entities: vec![],
            ais: RefCell::new(vec![]),
            stat_blocks: vec![],
            entity_views: vec![],
            entity_loots: vec![],
            resources: vec![
                rc_load("dude.png.xp"), //0
                rc_load("firelemental.xp"), //1
                rc_load("guard.xp"),  //2
                rc_load("player.xp"), //3
                rc_load("spider.xp"), //4
                rc_load("kingspider.xp"), //5
            ],
            queued_destruction: RefCell::new(vec![]),
            free_slots: vec![],

            camera: RefCell::new(Camera::new(-20, -20)),

            waiting_for_directional_input: false,
            directional_callback: None,
            currently_viewed_art: None,
            currently_viewed_stat_block: None,

            until_player_save: 10.0,

            portal_locations: vec![
                "output.map"
            ],
            destination_next_tick: RefCell::new(None),
        };

        let player_stat_block: StatBlock = {
            if Path::new("player.json").exists() {
                let mut file = File::open("player.json").unwrap();
                let mut string_buf = String::new();
                file.read_to_string(&mut string_buf).unwrap();
                serde_json::from_str(&string_buf).unwrap()
            } else {
                StatBlock {
                    id: 0,
                    hp: EntityStat::new("Hit Points", 10),
                    def: EntityStat::new("Defense", 3),
                    atk: EntityStat::new("Attack", 5),
                    ..Default::default()
                }
            }
        };

        state.add_entity(
            0,
            Some(Box::new(RefCell::new(player))),
            Some(Box::new(PlayerAI)),
            Some(RefCell::new(player_stat_block)),
            Some(Rc::new(EntityView {
                name: "Me...".to_string(),
                art: state.resources[3].clone(),
            })),
        );

        Self::load_entities_from_map(&mut state, &load_map.entities);

        // state.generate_entities();

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
        (y as usize * self.map_width as usize) + x as usize
    }

    #[allow(dead_code)]
    fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.map_width, idx as i32 / self.map_width)
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.map_width && y >= 0 && y < self.map_height
    }

    fn can_move(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y)
            && match self.tiles[self.xy_idx(x, y)].get() {
                TileType::Wall(_) => false,
                _ => true,
            }
    }

    fn load_map_by_destination(&mut self, destination: usize, x: i32, y: i32) {
        // TFW you wish you were using an ECS :(

        *self.destination_next_tick.borrow_mut() = None;

        println!("{} {}", x, y);

        let x_map = self.portal_locations[destination];
        let load_map = map_utils::load_from_file(x_map);

        self.map_width = load_map.width;
        self.map_height = load_map.height;

        self.tiles = map_utils::map_to_cells(load_map.tiles);
        let mut _entity_0 = std::mem::take(&mut self.entities[0]);
        _entity_0.as_ref().unwrap().borrow_mut().set_x(x);
        _entity_0.as_ref().unwrap().borrow_mut().set_y(y);
        self.entities = vec![_entity_0];
        {
            let mut ais_arr = self.ais.borrow_mut();
            let _ais_0 = std::mem::take(&mut ais_arr[0]);
            *ais_arr = vec![_ais_0];
        }
        let _stat_block_0 = std::mem::take(&mut self.stat_blocks[0]);
        self.stat_blocks = vec![_stat_block_0];
        let _entity_view_0 = std::mem::take(&mut self.entity_views[0]);
        self.entity_views = vec![_entity_view_0];
        let _entity_loot_0 = std::mem::take(&mut self.entity_loots[0]);
        self.entity_loots = vec![_entity_loot_0];

        self.free_slots = vec![];

        Self::load_entities_from_map(self, &load_map.entities);

        self.camera.borrow_mut().update_xy(x, y);
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
            cm.dx += deltas.0;
            cm.dy += deltas.1;
        }
        let plyr = self.get_entity(0).borrow_mut();
        let (x, y) = (plyr.get_x(), plyr.get_y());
        let idx_of = self.xy_idx(x, y);
        if let TileType::Portal(_, destination, x, y) = self.tiles[idx_of].get() {
            *self.destination_next_tick.borrow_mut() = Some((destination, x, y));
        }
    }

    fn draw_map(&self, g_db: &mut DrawBatch) {
        let l_x = math_utils::clamp(self.camera.borrow().mod_x(), 0, self.map_width);
        let h_x = math_utils::clamp(self.camera.borrow().mod_x() + 40, 0, self.map_width);
        let l_y = math_utils::clamp(self.camera.borrow().mod_y(), 0, self.map_height);
        let h_y = math_utils::clamp(self.camera.borrow().mod_y() + 40, 0, self.map_height);

        for x in l_x..h_x {
            for y in l_y..h_y {
                let tile = self.tiles[self.xy_idx(x, y)].get();
                let (x, y) = self.camera.borrow().transform_point((x, y));

                match tile {
                    TileType::Floor(d) => {
                        g_db.set(Point::new(x, y), ColorPair::new(d.fg, d.bg), d.glyph);
                    }
                    TileType::Wall(d) => {
                        g_db.set(Point::new(x, y), ColorPair::new(d.fg, d.bg), d.glyph);
                    }
                    TileType::Portal(d, _, _, _) => {
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

    fn save_player(&self) {
        let mut file = File::create("player.json").unwrap();
        let string_buf = serde_json::to_string(&self.stat_blocks[0].unwrap_ref().clone()).unwrap();
        file.write(string_buf.as_bytes()).expect("Failed to write to player.json");
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {

        let destination_tick_info = {
            if let Some((destination, x, y)) = *self.destination_next_tick.borrow_mut() {
                Some((destination, x, y))
            } else {
                None
            }
        };

        if let Some((destination, x, y)) =  destination_tick_info {
            self.load_map_by_destination(destination, x, y)
        }

        let mut g_db = DrawBatch::new();
        g_db.cls();

        self.camera.borrow_mut().tween_tick(ctx.frame_time_ms);

        self.until_player_save -= ctx.frame_time_ms / 1000.0;
        if self.until_player_save <= 0.0 {
            self.until_player_save = 30.0;
            self.save_player();
        }

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
            let mut stat_block_to_draw = if !self.currently_viewed_stat_block.is_none() {
                self.currently_viewed_stat_block.as_ref().unwrap().clone()
            } else {
                self.stat_blocks[0].unwrap_ref().clone()
            };

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
                        self.currently_viewed_stat_block = Some(stat_block.borrow().clone());
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
        } else {
            let player_art = self.entity_views[0].as_ref().unwrap();
            self.print_image_at(41, 20, player_art, ctx)
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

    //Ask the user for a number and then get it from stdin
    let mut gametype = String::new();
    println!("Type 0 for normal game 1 for map editor");
    std::io::stdin().read_line(&mut gametype)?;
    let gametype = gametype.trim().parse::<i32>().unwrap();

    if gametype == 0 {
        let gs = State::new();
        rltk::main_loop(context, gs)
    } else {
        let gs = MapEditorState::new(32, 32);
        rltk::main_loop(context, gs)
    }
}
