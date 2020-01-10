#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Simple(SimpleTile),
    Bonus(BonusTile),
    Honor(HonorTile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Coins,
    Bamboo,
    Characters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleTile {
    pub number: u8,
    pub suit: Suit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BonusTile {
    Flower(Flower),
    Season(Season),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flower {
    PlumBlossom,
    Orchid,
    Chrysanthemum,
    Bamboo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HonorTile {
    Wind(Wind),
    Dragon(Dragon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dragon {
    Red,
    Green,
    White,
}
