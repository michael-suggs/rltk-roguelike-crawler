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
