use serde::{Deserialize, Serialize, de::DeserializeOwned};
use specs::{
    prelude::*,
    saveload::{Marker, ConvertSaveload},
    error::NoError,
};
use specs_derive::*;
use rltk::{RGB};

/// Component detailing the 2D position of an entity.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Component for entities that can be rendered to the screen.
///
/// Entities will be rendered as their glyph, with said glyph having color `fg`
/// laid over a background of color `bg`.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

/// Component used to "tag" player entities.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

/// Component encapsulating the visible range of the current entity.
///
/// A viewshed is a cartographical term that literally translates to "what I can
/// see from here." A vector of `visible_tiles` holds map points that refer to
/// all tiles that are visible to the current entity from their current position,
/// and the integer `range` details the field-of-view of the current entity.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

/// Component tag indicating entity is a monster.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Monster {}

/// Allows for naming of entities.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

/// Component blocks its inhabited tile.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

/// Component holding combat stats for an entity.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

/// Flag: entity is able to act at range.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32
}

/// Entity with this can pass along confusion.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

/// Flag: entity affects others within radius of its location.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32
}

/// Flag: entity is able to inflict damage on other entities.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32
}

/// Struct used for handling and applying damage to entities.
#[derive(Component, Debug, ConvertSaveload, Clone)]
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

/// Intent: entity wants to engage in melee combat against `target`.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity
}

/// Intent. Taken on when an entity tries to pick up an item.
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

/// Intent. Taken on when an entity tries to drop an item.
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity
}

/// Intent. Taken on when an entity tries to use an item.
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>
}

/// Flag: entity with this flag is in the possession (backpack) of `owner`.
#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity
}

/// Flag: an item.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

/// Flag: consumable item.
///
/// A component with this flag will be destroyed on use.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

/// Flag: an item that provides healing.
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32
}

#[derive(Component, Debug)]
pub struct SerializeMe;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
}
