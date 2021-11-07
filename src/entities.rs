
pub mod entity_create {
    use crate::*;

    fn basic_en(id: usize, pos: (i32, i32), glyph: u16, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> BasicEntity {
        BasicEntity {
            id,
            x: pos.0,
            y: pos.1,
            d: Display {
                glyph,
                fg: RGB::named(fg),
                bg: RGB::named(bg),
            }
        }
    }

    pub fn create_fire_elemental(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, '*' as u16, rltk::BLACK, rltk::RED);

        let ai_component = ZombieAI { id: entity_id };

        let stat_component = StatBlock {
            hp: 20,
            atk: 3,
            def: 1,
            id: entity_id,

            ..Default::default()
        };

        let goblin_man_art = state.resources[1].clone();
        state.add_entity(
            entity_id,
            Some(Box::new(RefCell::new(entity_component))),
            Some(Box::new(ai_component)),
            Some(RefCell::new(stat_component)),
            Some(Rc::new(EntityView {
                name: String::from("Minor Fire Elemental"),
                art: goblin_man_art.clone(),
            })),
        );
    }

    pub fn create_goblin(state: &mut State, pos: (i32, i32)) {
        let entity_id = state.consume_free_slot();

        let entity_component = basic_en(entity_id, pos, 'g' as u16, rltk::RED, rltk::BLACK);

        let ai_component = ZombieAI { id: entity_id };

        let stat_component = StatBlock {
            hp: 10,
            atk: 1,
            id: entity_id,

            ..Default::default()
        };

        let goblin_man_art = state.resources[0].clone();
        state.add_entity(
            entity_id,
            Some(Box::new(RefCell::new(entity_component))),
            Some(Box::new(ai_component)),
            Some(RefCell::new(stat_component)),
            Some(Rc::new(EntityView {
                name: String::from("Goblin Man"),
                art: goblin_man_art.clone(),
            })),
        );
    }
}
