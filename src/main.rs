extern crate rltk;
use std::{cell::{Cell, RefCell, RefMut}, cmp::max};

use rltk::{BResult, BTermBuilder, GameState, RGB, Rltk, VirtualKeyCode};

mod math_utils {
    use rand::Rng;

    pub fn chance(f: f32) -> bool {
        return rand::thread_rng().gen::<f32>() < f;
    }

}

#[derive(Copy, Clone, PartialEq)]
enum TileType {
    Wall,
    Floor,
    Object(Display),
}

#[derive(Copy, Clone, PartialEq)]
struct Display {
    glyph: u16,
    fg: RGB,
    bg: RGB,
}

trait EntityAI {
    fn on_turn(&mut self, me: RefMut<dyn Entity>, state: &State);
    fn on_remove(&mut self, me: RefMut<dyn Entity>, state: &State);
    fn on_death(&mut self, me: RefMut<dyn Entity>, state: &State);
}

trait Entity {
    fn get_x(&self) -> i32;
    fn get_y(&self) -> i32;
    fn get_display(&self) -> Display;

    //May need to mutate the ai and therefore self
    fn get_ai(&mut self) -> Option<&mut Box<dyn EntityAI>>;

    fn set_x(&mut self, x: i32);
    fn set_y(&mut self, y: i32);
    fn set_display(&mut self, display: Display);
}

struct NullEntity;
impl Entity for NullEntity {
    fn get_x(&self) -> i32 {
        0
    }
    fn get_y(&self) -> i32 {
        0
    }
    fn get_display(&self) -> Display {
        Display {
            glyph: 0,
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
        }
    }
    fn get_ai(&mut self) -> Option<&mut Box<dyn EntityAI>> {
        None
    }
    fn set_x(&mut self, _x: i32) {}
    fn set_y(&mut self, _y: i32) {}
    fn set_display(&mut self, _display: Display) {}
}

struct BasicEntity {
    x: i32,
    y: i32,
    d: Display,
}

struct AIEntity {
    en: BasicEntity,
    ai: Box<dyn EntityAI>,
}

struct PlayerEntity {
    x: i32,
    y: i32,
    d: Display,

    atk: i32,
    hp: i32,
}

impl PlayerEntity {

}

impl Entity for AIEntity {
    fn get_x(&self) -> i32 {
        self.en.x
    }

    fn get_y(&self) -> i32 {
        self.en.y
    }

    fn get_display(&self) -> Display {
        self.en.d
    }

    fn get_ai(&mut self) -> Option<&mut Box<dyn EntityAI>> {
        Some(&mut self.ai)
    }

    fn set_x(&mut self, x: i32) {
        self.en.x = x;
    }

    fn set_y(&mut self, y: i32) {
        self.en.y = y;
    }

    fn set_display(&mut self, display: Display) {
        self.en.d = display;
    }
}

struct NullAI;
impl EntityAI for NullAI {
    fn on_turn(&mut self, _: RefMut<dyn Entity>, _: &State) {
        println!("THIS SHOULDN'T PRINT, NULLAI TICK");
    }
    fn on_remove(&mut self, _: RefMut<dyn Entity>, _: &State) {}
    fn on_death(&mut self, _: RefMut<dyn Entity>, _: &State) {}
}

struct ZombieAI;
impl EntityAI for ZombieAI {
    fn on_turn(&mut self, me: RefMut<dyn Entity>, state: &State) {
        if math_utils::chance(0.5) { return; }

        let player_pos = (state.entities[0].borrow().get_x(), state.entities[0].borrow().get_y());
        let zombie_pos = (me.get_x(), me.get_y());
        //Calculate the direction to the player from zombie_pos
        let dx = player_pos.0 - zombie_pos.0;
        let dy = player_pos.1 - zombie_pos.1;
        //Normalize dx, dy

        let dx = dx / max(1, dx.abs());
        let dy = dy / max(1, dy.abs());

        state.move_entity_by(me, dx, dy);
    }
    fn on_remove(&mut self, _: RefMut<dyn Entity>, _: &State) {}
    fn on_death(&mut self, _: RefMut<dyn Entity>, _: &State) {}
}

impl Entity for BasicEntity {
    fn get_x(&self) -> i32 {
        self.x
    }
    fn get_y(&self) -> i32 {
        self.y
    }
    fn get_display(&self) -> Display {
        self.d
    }
    fn get_ai(&mut self) -> Option<&mut Box<dyn EntityAI>> {
        None
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
}

struct Camera {
    x: i32,
    y: i32,
}

impl Camera {
    fn transform_point(&self, point: (i32, i32)) -> (i32, i32) {
        (point.0 - self.x, point.1 - self.y)
    }
}

struct State {
    entities: Vec<Box<RefCell<dyn Entity>>>,
    tiles: Vec<Cell<TileType>>,
    camera: Camera,

    waiting_for_directional_input: bool
}

impl State {

    fn on_turn(&mut self) {
        for e in self.entities.iter() {
            let mut ai = {
                match e.borrow_mut().get_ai() {
                    Some(ai) => std::mem::replace(ai, Box::new(NullAI)),
                    None => continue,
                }
            };

            ai.on_turn(e.borrow_mut(), self);
            //Return the memory
            let _ = std::mem::replace(e.borrow_mut().get_ai().unwrap(), ai);
        }
    }

    fn generate_map() -> Vec<Cell<TileType>> {
        let mut map = vec![];
        for _ in 0..80*50 {
            if math_utils::chance(0.03) {
                map.push(Cell::new(TileType::Wall));
            } else {
                map.push(Cell::new(TileType::Floor));
            }
        }
        map
    }

    fn new() -> State {

        let player = BasicEntity {
            x: 1,
            y: 1,
            d: Display {
                glyph: '@' as u16,
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        };

        let mut entities: Vec<Box<RefCell<dyn Entity>>> = Vec::new();

        let basic_enemy = AIEntity {
            en: BasicEntity {
                x: 5,
                y: 5,
                d: Display {
                    glyph: 'z' as u16,
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                },
            },
            ai: Box::new(ZombieAI),
        };
        
        let boxcell = |b| {
            Box::new(RefCell::new(b))
        };

        entities.push(boxcell(player));
        entities.push(Box::new(RefCell::new(basic_enemy)));

        let mut state = State {
            tiles: State::generate_map(),
            entities,
            camera: Camera {
                x: -20,
                y: -20,
            },
            
            waiting_for_directional_input: false
        };

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

    fn move_entity(mut entity: RefMut<dyn Entity>, x: i32, y: i32) {
        entity.set_x(x);
        entity.set_y(y);
    }

    fn move_entity_by(&self, entity: RefMut<dyn Entity>, x: i32, y: i32) {
        let new_x = entity.get_x() + x;
        let new_y = entity.get_y() + y;
        if self.can_move(new_x, new_y) {
            State::move_entity(entity, new_x, new_y);
        }
    }

    fn move_player_by(&mut self, x: i32, y: i32) {
        let new_x = x + self.entities[0].borrow().get_x();
        let new_y = y + self.entities[0].borrow().get_y();
        if self.can_move(new_x, new_y) {
            State::move_entity(self.entities[0].borrow_mut(), new_x, new_y);
            self.camera.x += x;
            self.camera.y += y;
        }
    }

    fn draw_map(&self, ctx: &mut Rltk) {
        for (i, tile) in self.tiles.iter().enumerate() {
            let (x, y) = self.idx_xy(i);
            let (x, y) = self.camera.transform_point((x, y));
            match tile.get() {
                TileType::Floor => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0.5, 0.5, 0.5),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('.'),
                    );
                }
                TileType::Wall => {
                    ctx.set(
                        x,
                        y,
                        RGB::from_f32(0., 1.0, 0.),
                        RGB::from_f32(0., 0., 0.),
                        rltk::to_cp437('#'),
                    );
                }
                TileType::Object(d) => {
                    ctx.set(x, y, d.fg, d.bg, d.glyph);
                }
            }
        }
        for entity in self.entities.iter() {
            let entity = entity.borrow();
            let (x, y) = (entity.get_x(), entity.get_y());
            let (x, y) = self.camera.transform_point((x, y));
            ctx.set(
                x,
                y,
                entity.get_display().fg,
                entity.get_display().bg,
                entity.get_display().glyph,
            );
        }

        //Draw directional arrows around player
        {
            let player = self.entities[0].borrow();
            let (x, y) = (player.get_x(), player.get_y());
            let (x, y) = self.camera.transform_point((x, y));

            const DRW_ARR: fn(i32, i32, char, &mut rltk::BTerm) = |x: i32, y: i32, c: char, ctx: &mut rltk::BTerm| {
                ctx.set(x, y, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), rltk::to_cp437(c));
            };

            //Draw the arrows using DRW_ARR around the player
            DRW_ARR(x+1, y+1, /*'↘'*/'\\', ctx);
            DRW_ARR(x-1, y+1, /*'↙'*/'/', ctx);
            DRW_ARR(x+1, y-1, /*'↗'*/'/', ctx);
            DRW_ARR(x-1, y-1, /*'↖'*/'\\', ctx);
            //The top left and right corners
            DRW_ARR(x, y-1, '↑', ctx);
            DRW_ARR(x, y+1, '↓', ctx);
            DRW_ARR(x+1, y, '→', ctx);
            DRW_ARR(x-1, y, '←', ctx);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        //Handle keyboard input WASD movements using self.move_player
        match ctx.key {
            None => {}
            Some(key) => {
                let mut do_tick = true;
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
                        do_tick = false;
                    }
                    _ => {}
                };  
                if do_tick {
                    self.on_turn();
                }
            },
        }

        self.draw_map(ctx);
    }
}

fn main() -> BResult<()> {
    let context = BTermBuilder::simple(40, 40).unwrap().build()?;
    let gs = State::new();
    rltk::main_loop(context, gs)
}
