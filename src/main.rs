use rltk::{GameState, Point, Rltk};
use specs::{prelude::*, saveload::{SimpleMarker, SimpleMarkerAllocator}};

extern crate serde;

mod components;
pub use components::*;
mod damage_system;
pub use damage_system::*;
mod gamelog;
pub use gamelog::*;
mod gui;
pub use gui::*;
mod inventory_system;
pub use inventory_system::*;
mod map;
pub use map::*;
mod map_indexing_system;
pub use map_indexing_system::*;
mod melee_combat_system;
pub use melee_combat_system::*;
mod monster_ai_system;
pub use monster_ai_system::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod saveload_system;
use saveload_system::*;
mod spawner;
pub use spawner::*;
mod visibility_system;
pub use visibility_system::VisibilitySystem;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: gui::MainMenuSelection },
    SaveGame,
    NextLevel,
}

fn main () -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    // Give it a retro vibe, because that's cool.
    context.with_post_scanlines(true);

    let mut gs = State{ ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map: Map = Map::new_map_rooms_and_corridors(1);
    let (player_x, player_y) = map.rooms[0].center();

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);

    // Generate some monsters.
    // Rolls dice to determine monster type, with orcs having glyph
    // `o` and goblins having glyph `g`.
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    // Gives a readily accessible handle on the player and their position.
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    // Init the game log, inserting as a resource.
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike!".to_string()]
    });
    // Game starts in prerun state to set up systems before beginning.
    gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });

    rltk::main_loop(context, gs)
}

pub struct State {
    pub ecs: World
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        let mut pickup = ItemCollectionSystem {};
        pickup.run_now(&self.ecs);
        let mut item_use = ItemUseSystem {};
        item_use.run_now(&self.ecs);
        let mut drop_items = ItemDropSystem {};
        drop_items.run_now(&self.ecs);
        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let player_ent = self.ecs.fetch::<Entity>();
        let backpack = self.ecs.read_storage::<InBackpack>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for ent in entities.join() {
            let mut should_delete = true;

            // Don't delete the player!
            if let Some(p) = player.get(ent) {
                should_delete = false;
            };

            if let Some(bp) = backpack.get(ent) {
                if bp.owner == *player_ent { should_delete = false; }
            };

            if should_delete { to_delete.push(ent); }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or their equipment.
        for target in self.entities_to_remove_on_level_change() {
            self.ecs.delete_entity(target).expect("Unable to delete entity");
        }

        // Build a new map for the next level.
        let worldmap = {
            let mut worldmap_res = self.ecs.write_resource::<Map>();
            *worldmap_res = Map::new_map_rooms_and_corridors(worldmap_res.depth + 1);
            worldmap_res.clone()
        };

        // Spawn some enemies.
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room);
        }

        // Place the player and update resources.
        let (player_x, player_y) = worldmap.rooms[0].center();
        let mut player_pos = self.ecs.write_resource::<Point>();
        *player_pos = Point::new(player_x, player_y);
        let player_ent = self.ecs.fetch::<Entity>();
        let mut pos_components = self.ecs.write_storage::<Position>();
        if let Some(player_pos_comp) = pos_components.get_mut(*player_ent) {
            player_pos_comp.x = player_x;
            player_pos_comp.y = player_y;
        }

        // Mark player visibility as dirty--entire map has changed!
        if let Some(vs) = self.ecs.write_storage::<Viewshed>().get_mut(*player_ent) {
            vs.dirty = true;
        }

        // Notify player of level change and give them a health boost.
        let mut log = self.ecs.fetch_mut::<gamelog::GameLog>();
        log.entries.push("You descend further into the depths, and take a moment to heal".to_string());
        if let Some(player_stats) = self.ecs.write_storage::<CombatStats>().get_mut(*player_ent) {
            player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }

        ctx.cls();

        match new_runstate {
            RunState::MainMenu{..} => {},
            _ => {
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                        }
                    }
                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        // RunState state machine
        match new_runstate {
            RunState::MainMenu{..} => {
                match gui::main_menu(self, ctx) {
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu { menu_selection: selected }
                    },
                    gui::MainMenuResult::Selected { selected } => {
                        match selected {
                            gui::MainMenuSelection::NewGame => new_runstate = RunState::PreRun,
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                new_runstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            },
                            gui::MainMenuSelection::Quit => { ::std::process::exit(0); },
                        }
                    }
                }
            },
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame
                };
            },
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            },
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            },
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::MonsterTurn;
            },
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            },
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_ent = result.1.unwrap();
                        if let Some(is_ranged) = self.ecs.read_storage::<Ranged>().get(item_ent) {
                            new_runstate = RunState::ShowTargeting {
                                range: is_ranged.range,
                                item: item_ent
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(*self.ecs.fetch::<Entity>(), WantsToUseItem {
                                    item: item_ent,
                                    target: None
                                })
                                .expect("Unable to insert intent");
                            new_runstate = RunState::PlayerTurn;
                        }
                    }
                }
            },
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_ent = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(), WantsToDropItem { item: item_ent })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            },
            RunState::ShowTargeting { range, item } => {
                let target = gui::ranged_target(self, ctx, range);
                match target.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(*self.ecs.fetch::<Entity>(),
                                    WantsToUseItem { item, target: target.1 })
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            },
            RunState::NextLevel => {
                self.goto_next_level();
                new_runstate = RunState::PreRun;
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        damage_system::delete_the_dead(&mut self.ecs);
    }
}
