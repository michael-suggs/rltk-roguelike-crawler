extern crate serde;

use rltk::{GameState, Point, Rltk};
use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator},
};

use damage_system::DamageSystem;
use hunger_system::HungerSystem;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem};
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use monster_ai_system::MonsterAI;
use particle_system::ParticleSpawnSystem;
use player::*;
use visibility_system::VisibilitySystem;

pub use components::*;
pub use map::*;
pub use map_builder::*;
pub use rect::Rect;

mod components;
mod damage_system;
mod gamelog;
mod gui;
mod hunger_system;
mod inventory_system;
mod map;
mod map_builder;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod particle_system;
mod player;
mod random_table;
mod rect;
mod rex_assets;
mod spawner;
mod trigger_system;
mod visibility_system;

pub mod saveload_system;

const SHOW_MAPGEN_VISUALIZER: bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    MagicMapReveal {
        row: i32,
    },
    GameOver,
    MapGeneration,
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    // Give it a retro vibe, because that's cool.
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

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
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(rex_assets::RexAssets::new());
    gs.ecs.insert(Map::new(1));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    let player_ent = spawner::player(&mut gs.ecs, 0, 0);
    gs.ecs.insert(player_ent);

    // Init the game log, inserting as a resource.
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike!".to_string()],
    });
    // Game starts in prerun state to set up systems before beginning.
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    // gs.ecs.insert(RunState::MainMenu { menu_selection: gui::MainMenuSelection::NewGame });
    gs.ecs.insert(RunState::MapGeneration {});

    gs.generate_world_map(1);

    rltk::main_loop(context, gs)
}

/// Handles game states and transitions.
pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    /// Runs all game systems on call, keeping things up to date.
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut triggers = trigger_system::TriggerSystem {};
        triggers.run_now(&self.ecs);
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
        let mut item_remove = ItemRemoveSystem {};
        item_remove.run_now(&self.ecs);
        let mut particles = ParticleSpawnSystem {};
        particles.run_now(&self.ecs);
        let mut hunger = HungerSystem {};
        hunger.run_now(&self.ecs);

        self.ecs.maintain();
    }

    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = map_builder::random_builder(new_depth, &mut rng);
        builder.build_map(&mut rng);
        std::mem::drop(rng);

        self.mapgen_history = builder.build_data.history.clone();
        let player_start = {
            let mut worldmap_res = self.ecs.write_resource::<Map>();
            *worldmap_res = builder.build_data.map.clone();
            builder.build_data.start.as_mut().unwrap().clone()
        };

        builder.spawn_entities(&mut self.ecs);
        {
            let mut player_position = self.ecs.write_resource::<Point>();
            *player_position = Point::new(player_start.x, player_start.y);
        }
        {
            let mut position_components = self.ecs.write_storage::<Position>();
            let player_ent = self.ecs.fetch::<Entity>();
            if let Some(player_pos_comp) = position_components.get_mut(*player_ent) {
                player_pos_comp.x = player_start.x;
                player_pos_comp.y = player_start.y;
            }

            let mut viewshed_comps = self.ecs.write_storage::<Viewshed>();
            if let Some(vs) = viewshed_comps.get_mut(*player_ent) {
                vs.dirty = true;
            }
        }
    }

    /// Calculates what entities should be removed from the game when moving to a new level.
    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let player_ent = self.ecs.fetch::<Entity>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for ent in entities.join() {
            let mut should_delete = true;

            // Don't delete the player!
            if let Some(_) = player.get(ent) {
                should_delete = false;
            };

            // Don't clear the player's backpack
            if let Some(bp) = backpack.get(ent) {
                if bp.owner == *player_ent {
                    should_delete = false;
                }
            };

            // Don't delete the player's equipped items
            if let Some(eq) = equipped.get(ent) {
                if eq.owner == *player_ent {
                    should_delete = false;
                }
            }

            // If none of the above, safe to delete
            if should_delete {
                to_delete.push(ent);
            }
        }

        to_delete
    }

    /// When using a staircase, generates a new level and sends the player to it.
    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or their equipment.
        for target in self.entities_to_remove_on_level_change() {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        let new_depth = {
            let worldmap_res = self.ecs.fetch::<Map>();
            worldmap_res.depth + 1
        };
        self.generate_world_map(new_depth);

        // Notify player of level change and give them a health boost.
        let player_ent = self.ecs.fetch::<Entity>();
        let mut log = self.ecs.fetch_mut::<gamelog::GameLog>();
        log.entries
            .push("You descend further into the depths, and take a moment to heal".to_string());
        if let Some(player_stats) = self.ecs.write_storage::<CombatStats>().get_mut(*player_ent) {
            player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);
        }
    }

    /// Cleans up resources and storage after a game over event, and sets up for a new game.
    fn game_over_cleanup(&mut self) {
        // Delete all game entities in preparation for new ones.
        let mut to_delete: Vec<Entity> = Vec::new();
        self.ecs.entities().join().for_each(|e| to_delete.push(e));
        to_delete
            .iter()
            .for_each(|e| self.ecs.delete_entity(*e).expect("Deletion failed"));

        {
            // Create a new player and get their intended location.
            let player_ent = spawner::player(&mut self.ecs, 0, 0);
            let mut player_ent_writer = self.ecs.write_resource::<Entity>();
            *player_ent_writer = player_ent;
        }

        self.generate_world_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        // Fetch and get a handle to our current runstate.
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        // Clear the active console.
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        // Keeps the system from rendering the map behind the main menu.
        match new_runstate {
            RunState::MainMenu { .. } => {}
            // If we're not at the main menu, go ahead and render the map.
            RunState::GameOver { .. } => {}
            _ => {
                draw_map(&self.ecs.fetch::<Map>(), ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let hidden = self.ecs.read_storage::<Hidden>();
                    let map = self.ecs.fetch::<Map>();

                    // Sort our renderables to allow for a rendering order.
                    let mut data = (&positions, &renderables, !&hidden)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));

                    // Visible tiles.
                    for (pos, render, _) in data.iter() {
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
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    new_runstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                draw_map(&self.mapgen_history[self.mapgen_index], ctx);

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        new_runstate = self.mapgen_next_state.unwrap();
                    }
                }
            }
            // At the main menu
            RunState::MainMenu { .. } => {
                match gui::main_menu(self, ctx) {
                    // Stay at the main menu until an option is selected.
                    gui::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    // When the player has selected a menu option, perform its action.
                    gui::MainMenuResult::Selected { selected } => {
                        match selected {
                            // Start up a new game
                            gui::MainMenuSelection::NewGame => new_runstate = RunState::PreRun,
                            // Try to load a saved game, and resume play.
                            gui::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                new_runstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            }
                            // Quits the game
                            gui::MainMenuSelection::Quit => {
                                ::std::process::exit(0);
                            }
                        }
                    }
                }
            }
            // Saves the game in its current state.
            RunState::SaveGame => {
                // Makes a savegame file and saves to it.
                saveload_system::save_game(&mut self.ecs);
                // Send the player back to the main menu on save.
                new_runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }
            // Tells the system to run all systems before starting the game.
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            // Waiting for the player to do something
            RunState::AwaitingInput => {
                new_runstate = player_input(self, ctx);
            }
            // Player has chosen an action--perform their chosen action by updating all systems.
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        new_runstate = RunState::MagicMapReveal { row: 0 };
                    }
                    _ => new_runstate = RunState::MonsterTurn,
                }
            }
            // Monster's turn to act.
            RunState::MonsterTurn => {
                // Monster action is handled by the AI, so just run the systems.
                self.run_systems();
                self.ecs.maintain();
                new_runstate = RunState::AwaitingInput;
            }
            // Open the inventory screen and handle inventory actions.
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    // Pressed escape--just close the inventory and wait for some other input.
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    // Haven't selected yet--loop here until something is chosen.
                    gui::ItemMenuResult::NoResponse => {}
                    // Selected something from the inventory.
                    gui::ItemMenuResult::Selected => {
                        let item_ent = result.1.unwrap();
                        // If the selected item has the `Ranged` component, send to targeting mode.
                        if let Some(is_ranged) = self.ecs.read_storage::<Ranged>().get(item_ent) {
                            new_runstate = RunState::ShowTargeting {
                                range: is_ranged.range,
                                item: item_ent,
                            };
                        } else {
                            // If not ranged, use the selected item.
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_ent,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            // Counts as player action--run systems in `PlayerTurn` to make
                            // effects of the item take place.
                            new_runstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            // Open the menu for dropping items from the player's inventory.
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    // Pressed escape--exit the menu and wait for another input from the player.
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    // Haven't selected anything yet--loop here until we have a selection.
                    gui::ItemMenuResult::NoResponse => {}
                    // Selected an item to drop.
                    gui::ItemMenuResult::Selected => {
                        // Insert intent to drop the selected item so the game's systems drop it.
                        let item_ent = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_ent },
                            )
                            .expect("Unable to insert intent");
                        // Systems handle dropping the item on the player's turn.
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            // Open the menu for removing equipped items.
            RunState::ShowRemoveItem => {
                let result = gui::remove_item_menu(self, ctx);
                match result.0 {
                    // Pressed escape--exit the menu and wait for another input from the player.
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    // Haven't selected anything yet--loop here until we have a selection.
                    gui::ItemMenuResult::NoResponse => {}
                    // Selected a piece of equipment to remove.
                    gui::ItemMenuResult::Selected => {
                        // Insert intent to remove the selected item so the game can take it off.
                        let item_ent = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_ent },
                            )
                            .expect("Unable to insert intent");
                        // Systems handle removing the item on the player's turn.
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            // Player has selected a ranged item--show the targeting interface.
            RunState::ShowTargeting { range, item } => {
                // Target is the tile selected by the player through the targeting interface.
                let target = gui::ranged_target(self, ctx, range);
                match target.0 {
                    // Pressed escape--exit targeting and wait for another input from the player.
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    // Haven't selected anything yet--loop here until we have a selection.
                    gui::ItemMenuResult::NoResponse => {}
                    // Selected a target.
                    gui::ItemMenuResult::Selected => {
                        // Insert intent to use the ranged item.
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: target.1,
                                },
                            )
                            .expect("Unable to insert intent");
                        // Systems handle using the item on the player's turn.
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            // Went down some stairs.
            RunState::NextLevel => {
                // Make a new map for the new depth level and send the player to it.
                self.goto_next_level();
                // PreRun on the new level to set everything up and in motion.
                new_runstate = RunState::PreRun;
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                (0..MAPWIDTH)
                    .map(|x| map.xy_idx(x as i32, row))
                    .collect::<Vec<usize>>()
                    .iter()
                    .for_each(|idx| map.revealed_tiles[*idx] = true);

                if row as usize == MAPHEIGHT - 1 {
                    new_runstate = RunState::MonsterTurn;
                } else {
                    new_runstate = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            // Player died.
            RunState::GameOver => match gui::game_over(ctx) {
                gui::GameOverResult::NoSelection => {}
                gui::GameOverResult::QuitToMenu => {
                    self.game_over_cleanup();
                    new_runstate = RunState::MainMenu {
                        menu_selection: gui::MainMenuSelection::NewGame,
                    };
                }
            },
        }

        // Set the game's state to the new state result from the match above.
        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = new_runstate;
        }

        // Remove dead entities to keep them from clogging up the map.
        damage_system::delete_the_dead(&mut self.ecs);
    }
}
