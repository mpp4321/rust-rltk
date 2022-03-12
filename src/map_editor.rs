use std::{fs::File, io::{stdin, stdout}};
use std::io::Write;

use rltk::{Rltk, GameState, VirtualKeyCode};
use serde::{Deserialize, Serialize};
use crate::structs::{map_utils::MapDescriptor, Display, TileType, self};

//Describes an entity in the map editor
#[derive(Clone, Serialize, Deserialize)]
pub struct MEEntity {
    pub d: Display,
    pub name: String,
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
    pub fn new(width: i32, height: i32) -> Self {
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
