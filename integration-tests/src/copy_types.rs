//! Example types that derive `Copy` and should therefore be marshaled by value.

use cs_bindgen::prelude::*;

#[cs_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleTile {
    pub suit: Suit,
    pub value: u8,
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Bamboo,
    Circles,
    Man,
}

#[cs_bindgen]
pub fn roundtrip_simple_tile(tile: SimpleTile) -> SimpleTile {
    tile
}
