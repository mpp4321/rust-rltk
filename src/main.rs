extern crate rltk;

mod entities;
mod structs;

use entities::entity_create;
use hecs::{Ref, RefMut, World};
use structs::*;

use serde::{Deserialize, Serialize};
use structs::map_utils::MapDescriptor;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::Path;
use std::sync::Arc;

use rltk::{
    BResult, BTermBuilder, ColorPair, DrawBatch, GameState, Point, Rltk, TextBlock, TextBuilder,
    VirtualKeyCode, XpFile, RGB,
};

pub type EntityIndex = hecs::Entity;

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

    #[allow(dead_code)]
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
                    _ => Display {
                        glyph: '?' as u16,
                        fg: rltk::BLACK,
                        bg: rltk::RED,
                    },
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

// Main game state
pub struct State {

    ecs: EntityWorld,

    resources: Vec<Arc<XpFile>>,

    queued_destruction: RefCell<Vec<EntityIndex>>,

    camera: RefCell<Camera>,

    waiting_for_directional_input: bool,
    directional_callback: Option<usize>,

    currently_viewed_art: Option<EntityIndex>,
    currently_viewed_stat_block: Option<EntityIndex>,

    until_player_save: f32,

    destination_next_tick: RefCell<Option<(usize, i32, i32)>>,

    open_window: Option<Box<dyn UInterface>>,
    
    map_state: InternalMapState,

}

impl State {

    pub fn close_open_window(&mut self) {
        self.open_window = None;
    }

    pub fn print_image_at(&self, x: i32, y: i32, entity_view: &EntityView, ctx: &mut Rltk) {
        ctx.render_xp_sprite(&entity_view.art, x, y);
    }

    pub fn queue_destruction(&self, slot: EntityIndex) {
        self.queued_destruction.borrow_mut().push(slot);
    }

    pub fn dispose_slot(&mut self, slot: EntityIndex) {
        self.ecs.despawn(slot).expect("Failed to dispose of entity");
    }

    fn handle_directional_input(&mut self, key: VirtualKeyCode) -> bool {
        if !self.waiting_for_directional_input {
            return false;
        }
        if self.directional_callback.is_none() {
            self.waiting_for_directional_input = false;
            return false;
        }
        let mut input_dir: Option<(i32, i32)> = None;
        match key {
            VirtualKeyCode::Up => input_dir = Some( (0, -1) ),
            VirtualKeyCode::Down => input_dir = Some( (0, 1) ),
            VirtualKeyCode::Left => input_dir = Some( (-1, 0) ),
            VirtualKeyCode::Right => input_dir = Some( (1, 0) ),

            VirtualKeyCode::Numpad8 => input_dir = Some( (0, -1) ),
            VirtualKeyCode::Numpad2 => input_dir = Some( (0, 1) ),
            VirtualKeyCode::Numpad4 => input_dir = Some( (-1, 0) ),
            VirtualKeyCode::Numpad6 => input_dir = Some( (1, 0) ),

            VirtualKeyCode::Numpad7 => input_dir = Some( (-1, -1) ),
            VirtualKeyCode::Numpad9 => input_dir = Some( (1, -1) ),
            VirtualKeyCode::Numpad1 => input_dir = Some( (-1, 1) ),
            VirtualKeyCode::Numpad3 => input_dir = Some( (1, 1) ),
            _ => {}
        };
        if input_dir.is_some() {
            let (dx, dy) = input_dir.unwrap();
            if let Some(slot) = self.directional_callback {
                let (px, py) = (
                    self.ecs.get_player().get_x() + dx,
                    self.ecs.get_player().get_y() + dy,
                );

                let mut found_entity: Option<EntityIndex> = None;

                for (entity, query) in self.ecs.query::<(&BasicEntity, &mut StatBlock)>()
                        .iter() {
                    let (ex, ey) = (query.0.get_x(), query.0.get_y());
                    if ex == px && ey == py {
                        found_entity = Some(entity);
                    }
                }

                let player_id = self.ecs.get_player_id();

                let player_container = self.ecs.get::<Container>(player_id).unwrap();

                let eff_chain_valid = { 
                    let player_equips = self.ecs.get::<Equipment>(player_id).unwrap();

                    player_equips.equips
                        .get(slot)
                        .and_then(|x|
                            x.and_then(|x| if player_container.items.get(x).is_some() { Some(x) } else { None } )
                        )
                        //.and_then(|a| a.effect_chain.as_ref().and_then(|z| Some(z)))
                };

                if let Some(eff_chain_idx) = eff_chain_valid {
                    let item_ref = player_container.items.get(eff_chain_idx).unwrap();
                    if let Some(eff_chain) = item_ref.effect_chain.clone().as_ref() {
                        drop(player_container);
                        eff_chain.handle_effect(&mut self.ecs, player_id, found_entity.iter().map(|z| *z).collect());
                    }
                }
            }
            self.directional_callback = None;
        }

        return true;
    }

    fn handle_movement_input(&mut self, key: VirtualKeyCode, do_tick: &mut bool) {

        if enumerate_keys().contains(&key) {
            self.waiting_for_directional_input = true;
            *do_tick = false;
            self.directional_callback = Some(get_index_from_key(key).expect("This should never run."));
            return;
        }

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
            _ => {
                *do_tick = false;
            }
        };
    }

    fn on_turn(&mut self) {
        //Copy ids out of query then run the system on them
        let _zombie_ticks = self.ecs
            .query::<&ZombieAI>()
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        let _self_destructs = self.ecs
            .query::<&SelfDestructAI>()
            .into_iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();


        for e in _zombie_ticks {
            ZombieAI::on_turn(self, e);
        }

        for e in _self_destructs {
            SelfDestructAI::on_turn(self, e);
        }

        PlayerAI::on_turn(self, self.ecs.get_player_id());

        let _dead_entities = self.ecs.query::<&StatBlock>().into_iter().filter(|( _, z )| {
            z.hp.get_total() <= 0
        }).map(|(e, _)| e).collect::<Vec<_>>();

        // handle deaths
        for e in _dead_entities {
            if self.ecs.get::<StatBlock>(e).unwrap().dead { return; }

            self.ecs.get_mut::<StatBlock>(e).unwrap().dead = true;

            self.ecs.insert_one(e, SelfDestructAI { turns_left: 10 }).expect("Failed to insert self destruct ai");

            self.ecs.get_mut::<BasicEntity>(e).unwrap().d = Display {
                glyph: rltk::to_cp437('%'),
                fg: rltk::RED,
                bg: rltk::BLACK
            };

        }


    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn generate_entities(&mut self) {
        for _ in 0..(self.map_width() * self.map_height()) {
            if !math_utils::chance(0.01) {
                continue;
            }

            let pos = math_utils::random_point(1, self.map_width() - 1, 1, self.map_height() - 1);

            if !math_utils::chance(0.3) {
                entity_create::create_goblin(self, pos);
            } else {
                entity_create::create_fire_elemental(self, pos);
            }
        }
    }

    fn load_entities_from_map(state: &mut State, entities: &Vec<Option<MEEntity>>) {
        state.map_state.clear_entities();
        for load_entity in entities.iter().enumerate() {
            if !load_entity.1.is_some() {
                continue;
            }
            let (x, y) = state.map_state.idx_xy(load_entity.0);
            let m_entity = load_entity.1.as_ref().unwrap();
            let e_indx = entity_create::resolve_entity_string(state, (x, y), m_entity.name.as_str());
            state.map_state.strict_add_from_pos((x, y), e_indx);
        }
    }

    fn new() -> State {
        let player = BasicEntity {
            x: 1,
            y: 1,
            d: Display {
                glyph: '@' as u16,
                fg: rltk::YELLOW,
                bg: rltk::BLACK,
            },
        };

        let f_c_dir = std::env::current_dir().unwrap();

        let arc_load = |s: &'static str| {
            Arc::new(
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

        let load_map = map_utils::load_from_file("main.map");

        let mut state = State {
            ecs: EntityWorld { ecs: World::new() },

            map_state: InternalMapState {
                map_width: load_map.width,
                map_height: load_map.height,

                tiles: map_utils::map_to_cells(load_map.tiles),
                portal_locations: vec![
                    "main.map",
                    "portal_land.map"
                ],
                entities: vec![],
            },

            resources: vec![
                arc_load("dude.png.xp"), //0
                arc_load("firelemental.xp"), //1
                arc_load("guard.xp"),  //2
                arc_load("player.xp"), //3
                arc_load("spider.xp"), //4
                arc_load("kingspider.xp"), //5
                arc_load("rockdude.xp"), //6
                arc_load("tall dude.xp"), //7
                arc_load("crazyeyes.xp"), //8
            ],
            queued_destruction: RefCell::new(vec![]),

            camera: RefCell::new(Camera::new(-20, -20)),

            waiting_for_directional_input: false,
            directional_callback: None,
            currently_viewed_art: None,
            currently_viewed_stat_block: None,

            until_player_save: 10.0,

            destination_next_tick: RefCell::new(None),
            open_window: None,
        };

        let player_stat_block: StatBlock = {
            if Path::new("player.json").exists() {
                let mut file = File::open("player.json").unwrap();
                let mut string_buf = String::new();
                file.read_to_string(&mut string_buf).unwrap();
                serde_json::from_str(&string_buf).unwrap()
            } else {
                StatBlock {
                    hp: EntityStat::new("Hit Points", 10),
                    def: EntityStat::new("Defense", 3),
                    atk: EntityStat::new("Attack", 5),
                    ..Default::default()
                }
            }
        };

        let player_pos = player.pos();
        let player_entity_id = state.ecs.spawn((
        Container {
            items: vec![Item {
                name: "Rusty Sword".to_string(),
                art: state.resources[0].clone(),
                // Gotta reference count this bs because callbacks are dumb in rust fml otherwise
                // deal with the wrath of the dyanmic size clone trait hate
                effect_chain: Arc::new(Some(Box::new(SingleTargetEffects::DamageTarget(None, 5)))),
            }],
            max_items: 999,
        }, 
        Equipment {
            equips: vec![None, None, None],
            max_equips: 3,
        },
        Player,
        player,
        PlayerAI,
        player_stat_block,
        EntityView {
                name: "Me...".to_string(),
                art: state.resources[3].clone(),
            }
        ));

        //This clears entities so
        Self::load_entities_from_map(&mut state, &load_map.entities);
        state.map_state.strict_add_from_pos(player_pos, player_entity_id);

        // state.generate_entities();

        state
    }

    fn set_tile(&mut self, x: i32, y: i32, t: TileType) {
        let idx = self.map_state.xy_idx(x, y);
        self.map_state.tiles[idx].set(t);
    }


    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.map_width() && y >= 0 && y < self.map_height()
    }

    fn can_move(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y)
            && match self.map_state.tiles[self.map_state.xy_idx(x, y)].get() {
                TileType::Wall(_) => false,
                _ => true,
            }
    }

    fn load_map_by_destination(&mut self, destination: usize, x: i32, y: i32) {
        // TFW you wish you were using an ECS :(

        *self.destination_next_tick.borrow_mut() = None;

        println!("{} {}", x, y);

        let x_map = self.map_state.portal_locations[destination];
        let load_map = map_utils::load_from_file(x_map);

        self.map_state.map_width = load_map.width;
        self.map_state.map_height = load_map.height;

        self.map_state.tiles = map_utils::map_to_cells(load_map.tiles);

        let entities_to_drop = self.ecs.query::<Option<&Player>>().iter().map(| (e, plo) | (e.clone(), plo.is_none())).collect::<Vec<_>>();

				for q_d in entities_to_drop {
            let (e, is_player) = q_d;
            if is_player {
                self.ecs.despawn(e).expect("failed to destroy entity");
            }
				}

        Self::load_entities_from_map(self, &load_map.entities);

        self.camera.borrow_mut().update_xy(x, y);

        let _pid = self.ecs.get_player_id();
        let mut _p = self.ecs.get_player_be();
        _p.set_x(x);
        _p.set_y(y);
        let pos = self.map_state.xy_idx(x, y);
        self.map_state.entities[pos].insert(_pid);
        
    }


    fn move_entity_by(&mut self, entity: EntityIndex, x: i32, y: i32) -> (i32, i32) {
        let entity_r = self.ecs.get_entity_comp(entity);
        let new_x = entity_r.get_x() + x;
        let new_y = entity_r.get_y() + y;
        if self.can_move(new_x, new_y) {
            drop(entity_r);
            let mut entity_r = self.ecs.get_entity_comp_mut(entity);
            let entity_r_pos = entity_r.pos();

            //Remove from old position
            // Return false for remove
            assert!(!self.map_state.set_entity_from_pos(entity_r_pos, entity), "Desynced entity position");
            entity_r.set_x(new_x);
            entity_r.set_y(new_y);
            // Add to new position
            // first retrieve position again
            let entity_r_pos = entity_r.pos();
            //Should return true for add
            assert!(self.map_state.set_entity_from_pos(entity_r_pos, entity), "Desynced entity position");
            return (x, y);
        }
        return (0, 0);
    }

    fn move_player_by(&mut self, x: i32, y: i32) {
        let deltas = self.move_entity_by(self.ecs.get_player_id(), x, y);
        {
            let mut cm = self.camera.borrow_mut();
            cm.dx += deltas.0;
            cm.dy += deltas.1;
        }
        let (x, y) = {
            let plyr = self.ecs.get_player_be();
            (plyr.get_x(), plyr.get_y())
        };
        let idx_of = self.map_state.xy_idx(x, y);
        if let TileType::Portal(_, destination, x, y) = self.map_state.tiles[idx_of].get() {
            *self.destination_next_tick.borrow_mut() = Some((destination, x, y));
        }
    }

    fn draw_map(&self, g_db: &mut DrawBatch) {
        let l_x = math_utils::clamp(self.camera.borrow().mod_x(), 0, self.map_state.map_width);
        let h_x = math_utils::clamp(self.camera.borrow().mod_x() + 40, 0, self.map_state.map_width);
        let l_y = math_utils::clamp(self.camera.borrow().mod_y(), 0, self.map_state.map_height);
        let h_y = math_utils::clamp(self.camera.borrow().mod_y() + 40, 0, self.map_state.map_height);

        for x in l_x..h_x {
            for y in l_y..h_y {
                let tile_idx_local = self.map_state.xy_idx(x, y);
                let tile = self.map_state.tiles[tile_idx_local].get();
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

        for entity in self.ecs.query::<&BasicEntity>().iter().map(|( _, p )| p) {
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
            let player = self.ecs.get_player();
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

    fn get_player_stat_block(&self) -> RefMut<StatBlock> {
        self.ecs.get_mut::<StatBlock>(self.ecs.get_player_id()).expect("Failed to get player stat block.")
    }

    fn save_player(&self) {
        let mut file = File::create("player.json").unwrap();
        let string_buf = serde_json::to_string(&*self.get_player_stat_block()).unwrap();
        file.write(string_buf.as_bytes()).expect("Failed to write to player.json");
    }

    fn update(&mut self, ctx: &mut Rltk) {
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

        self.camera.borrow_mut().tween_tick(ctx.frame_time_ms);

        self.until_player_save -= ctx.frame_time_ms / 1000.0;
        if self.until_player_save <= 0.0 {
            self.until_player_save = 30.0;
            self.save_player();
        }
    }

    fn render_game_window(&mut self, ctx: &mut Rltk) {
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
            let e_id = self.currently_viewed_stat_block.unwrap_or(self.ecs.get_player_id());
            let stat_block_to_draw = if !self.currently_viewed_stat_block.is_none() {
                let fetch_res = self.ecs.get_mut::<StatBlock>(e_id);
                if fetch_res.is_ok() { fetch_res.unwrap() }
                else {
                    self.currently_viewed_stat_block = None;
                    self.get_player_stat_block()
                }
            } else {
                self.get_player_stat_block()
            };

            //Draw the stat block in a TextBlock

            let mut tb = TextBuilder::empty();

            if let Ok(en_view) = self.ecs.get::<EntityView>(e_id) {
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
            for (e_id, entity) in self.ecs.query::<&BasicEntity>().iter() {
                let (ex, ey) = (entity.get_x(), entity.get_y());
                let (ex, ey) = self.camera.borrow().transform_point((ex, ey));
                if ex == x && ey == y {
                    let view = self.ecs.get::<EntityView>(e_id);
                    let stat_bl = self.ecs.get::<StatBlock>(e_id);

                    if let Ok(_) = view {
                        self.currently_viewed_art = Some(e_id);
                    }

                    if let Ok(_) = stat_bl {
                        self.currently_viewed_stat_block = Some(e_id);
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
                    if key == VirtualKeyCode::I {
                        self.open_window = Some(Box::new(InventoryUI {
                            container_id: self.ecs.get_player_id(),
                            pending_item: RefCell::new(None)
                        }));
                    }
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

        let _player_art = self
            .ecs
            .query::<(&Player, &EntityView)>()
            .iter().map(|(_, (_, c))| { c }).next().unwrap();

        let c_view_art = self.ecs.get::<EntityView>(self.currently_viewed_art.unwrap_or(self.ecs.get_player_id()));
        if c_view_art.is_err() {
            self.currently_viewed_art = None;
        }
        self.print_image_at(41, 20, &c_view_art.unwrap_or(self.ecs.get_player_view()), ctx);
    }

    fn map_width(&self) -> i32 {
        return self.map_state.map_width;
    }

    fn map_height(&self) -> i32 {
        return self.map_state.map_height;
    }

}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        if self.open_window.is_some() {
            let bo_w = std::mem::take(&mut self.open_window);

            let close = bo_w.as_ref()
                .unwrap()
                .on_input(self, ctx.key);

            bo_w.as_ref().unwrap().render(ctx, self);
            if !close {
                let _ = std::mem::replace(&mut self.open_window, bo_w);
            }
        } else {
            self.update(ctx);
            self.render_game_window(ctx);
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
