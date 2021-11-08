use std::borrow::BorrowMut;

use crate::{EntityIndex, math_utils};

pub mod entity_create {
    use crate::*;

    macro_rules! somebox {
        ($($x:tt)*) => {
            Some(Box::new($($x)*))
        };
    }

    macro_rules! somerefcell {
        ($($x:tt)*) => {
            Some(RefCell::new($($x)*))
        };
    }

    macro_rules! somerc {
        ($($x:tt)*) => {
            Some(Rc::new($($x)*))
        };
    }

    pub fn resolve_entity_string(state: &mut State, pos: (i32, i32), str_e: &str) {
        match str_e {
            "Goblin" => create_goblin(state, pos),
            "SFElemental" => create_fire_elemental(state, pos),
            "Spider" => create_spider(state, pos),
            "KSpider" => create_king_spider(state, pos),
            _ => panic!("Entity not able to be resolved")
        }
    }

    fn basic_en(
        id: usize,
        pos: (i32, i32),
        glyph: u16,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) -> BasicEntity {
        BasicEntity {
            id,
            x: pos.0,
            y: pos.1,
            d: Display {
                glyph,
                fg,
                bg
            },
        }
    }

    pub fn create_fire_elemental(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, '*' as u16, rltk::BLACK, rltk::RED);

        let ai_component = ZombieAI { id: entity_id };

        let mut stat_component = StatBlock {
            id: entity_id,
            ..Default::default()
        };
        stat_component.hp.set(20);
        stat_component.atk.set(3);
        stat_component.def.set(1);

        let art = state.resources[1].clone();
        add_entity(
            state,
            entity_component,
            entity_id,
            ai_component,
            stat_component,
            somerc!(
                EntityView {
                    name: String::from("S Fire Elemental"),
                    art: art.clone()
                }
            )
        );
    }

    fn add_entity<E: 'static + Entity, A: 'static + EntityAI>(
        state: &mut State,
        entity_component: E,
        entity_id: usize,
        ai_component: A,
        stat_component: StatBlock,
        entity_view: Option<Rc<EntityView>>
    ) {
        state.add_entity(
            entity_id,
            somebox!(RefCell::new(entity_component)),
            somebox!(ai_component),
            somerefcell!(stat_component),
            entity_view,
        );
    }

    pub fn create_king_spider(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, 'S' as u16, rltk::BURLYWOOD, rltk::BROWN2);
        let ai_component = ZombieAI { id: entity_id };

        let mut stat_component = StatBlock {
            id: entity_id,
            ..Default::default()
        };

        stat_component.hp.set(24);
        stat_component.def.set(3);
        stat_component.atk.set(7);

        let goblin_man_art = state.resources[5].clone();

        add_entity(
            state,
            entity_component,
            entity_id,
            ai_component,
            stat_component,
            somerc!(
                EntityView {
                    name: String::from("King Spider"),
                    art: goblin_man_art.clone()
                }
            )
        );

        state.entity_loots[entity_id] = somebox!(RefCell::new(
            SpiderLoot {
                id: entity_id,
                max_atk: 10
            }
        ));
    }

    pub fn create_spider(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, 's' as u16, rltk::BURLYWOOD, rltk::BROWN1);
        let ai_component = ZombieAI { id: entity_id };

        let mut stat_component = StatBlock {
            id: entity_id,
            ..Default::default()
        };

        stat_component.hp.set(12);
        stat_component.def.set(1);
        stat_component.atk.set(1);

        let art = state.resources[4].clone();
        add_entity(
            state,
            entity_component,
            entity_id,
            ai_component,
            stat_component,
            somerc!(
                EntityView {
                    name: String::from("Spider"),
                    art: art.clone()
                }
            )
        );
        state.entity_loots[entity_id] = somebox!(RefCell::new(
            SpiderLoot {
                id: entity_id,
                max_atk: 5
            }
        ));
    }

    pub fn create_goblin(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, 'g' as u16, rltk::RED, rltk::BLACK);
        let ai_component = ZombieAI { id: entity_id };
        let mut stat_component = StatBlock {
            id: entity_id,
            ..Default::default()
        };
        stat_component.hp.set(10);
        stat_component.atk.set(1);

        let goblin_man_art = state.resources[0].clone();
        add_entity(
            state,
            entity_component,
            entity_id,
            ai_component,
            stat_component,
            somerc!(
                EntityView {
                    name: String::from("Goblin Dude"),
                    art: goblin_man_art.clone()
                }
            )
        );
    }
}
