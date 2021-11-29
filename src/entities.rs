
pub mod entity_create {
    use crate::*;

    pub fn resolve_entity_string(state: &mut State, pos: (i32, i32), str_e: &str) -> EntityIndex {
        match str_e {
            "Goblin" => create_goblin(state, pos),
            "SFElemental" => create_fire_elemental(state, pos),
            "Spider" => create_spider(state, pos),
            "KSpider" => create_king_spider(state, pos),
            "Crazy Eyes" => create_crazy_eyes(state, pos),
            "Tall Dude" => create_tall_dude(state, pos),
            "Rock" => create_rock(state, pos),
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

    pub fn create_crazy_eyes(state: &mut State, pos: (i32, i32)) -> EntityIndex {
        let entity_component = basic_en(pos, '%' as u16, rltk::PURPLE4, rltk::RED);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(22);
        stat_component.atk.set(10);
        stat_component.def.set(5);

        let art = state.resources[8].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "Crazyyyy Eyes".to_string(),
                    art: art
                }
        ))
    }

    pub fn create_tall_dude(state: &mut State, pos: (i32, i32)) -> EntityIndex {
        let entity_component = basic_en(pos, '|' as u16, rltk::PURPLE4, rltk::DARKGRAY);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(13);
        stat_component.atk.set(20);
        stat_component.def.set(1);

        let art = state.resources[7].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "Tall Dude!".to_string(),
                    art: art
                }
        ))
    }
    
    pub fn create_rock(state: &mut State, pos: (i32, i32)) -> EntityIndex {
        let entity_component = basic_en(pos, '0' as u16, rltk::GRAY56, rltk::DARKGRAY);
        let ai_component = ZombieAI;
        let mut stat_component = StatBlock::default();

        stat_component.hp.set(47);
        stat_component.atk.set(1);
        stat_component.def.set(1);

        let art = state.resources[6].clone();
        
        state.ecs.spawn((
                entity_component,
                ai_component,
                stat_component,
                EntityView {
                    name: "Dah Rock".to_string(),
                    art: art
                }
        ))
    }

    pub fn create_fire_elemental(state: &mut State, pos: (i32, i32)) -> EntityIndex {
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
        ))
    }

    pub fn create_king_spider(state: &mut State, pos: (i32, i32)) -> EntityIndex {
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
        ))
    }


    pub fn create_spider(state: &mut State, pos: (i32, i32)) -> EntityIndex {
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
        ))
    }

    pub fn create_goblin(state: &mut State, pos: (i32, i32)) -> EntityIndex {
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
        ))
    }
}
