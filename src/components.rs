use specs::prelude::*;
use specs_derive::*;
use rltk::{RGB};

/// Component detailing the 2D position of an entity.
#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Component for entities that can be rendered to the screen.
///
/// Entities will be rendered as their glyph, with said glyph having color `fg`
/// laid over a background of color `bg`.
#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

/// Component used to "tag" player entities.
#[derive(Component, Debug)]
pub struct Player {}

/// Component encapsulating the visible range of the current entity.
///
/// A viewshed is a cartographical term that literally translates to "what I can
/// see from here." A vector of `visible_tiles` holds map points that refer to
/// all tiles that are visible to the current entity from their current position,
/// and the integer `range` details the field-of-view of the current entity.
#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

/// Component tag indicating entity is a monster.
#[derive(Component)]
pub struct Monster {}

/// Allows for naming of entities.
#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

/// Component blocks its inhabited tile.
#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount: vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
    pub owner: Entity
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Potion {
    pub heal_amount: i32
}

#[derive(Component, Debug)]
pub struct WantsToDrinkPotion {
    pub potion: Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToDropItem {
    pub item: Entity
}
