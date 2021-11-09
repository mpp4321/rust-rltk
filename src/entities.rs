
pub mod entity_create {
    use crate::*;

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
        pos: (i32, i32),
        glyph: u16,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) -> BasicEntity {
        BasicEntity {
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
        let entity_component = basic_en(pos, '*' as u16, rltk::RED, rltk::BROWN2);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(15);
        stat_component.atk.set(7);

        let goblin_man_art = state.resources[1].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "S Fire Elemental".to_string(),
                    art: goblin_man_art
                }
        ));
    }

    pub fn create_king_spider(state: &mut State, pos: (i32, i32)) {
        let entity_component = basic_en(pos, 'S' as u16, rltk::BURLYWOOD, rltk::BROWN2);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(24);
        stat_component.atk.set(7);

        let goblin_man_art = state.resources[5].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "King Spider".to_string(),
                    art: goblin_man_art
                }
        ));
    }


    pub fn create_spider(state: &mut State, pos: (i32, i32)) {
        let entity_component = basic_en(pos, 's' as u16, rltk::BURLYWOOD, rltk::BROWN2);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(12);
        stat_component.atk.set(2);

        let goblin_man_art = state.resources[4].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "Spider".to_string(),
                    art: goblin_man_art
                }
        ));
    }

    pub fn create_goblin(state: &mut State, pos: (i32, i32)) {
        let entity_component = basic_en(pos, 'g' as u16, rltk::RED, rltk::BLACK);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(10);
        stat_component.atk.set(1);

        let goblin_man_art = state.resources[0].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "Goblina".to_string(),
                    art: goblin_man_art
                }
        ));
    }
}
