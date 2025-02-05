use rand::Rng;

//--------------------
// Structures
//--------------------
// Primary Pokemon Structure
#[derive(Debug, Default)]
pub struct Pokemon {
    // Identifying Information
    pub id: u32,
    pub name: String,
    pub form: String,
    pub type_1: PokemonType,
    pub type_2: PokemonType,

    // Stats
    pub level: u32,
}

impl Pokemon {

}

// Pokemon Base Structure
#[derive(Debug)]
pub struct PokemonBase {
    // Identifying Information
    pub id: u32,
    pub name: String,
    pub form: String,
    pub type_1: PokemonType,
    pub type_2: PokemonType,
}

// Stats Structure
#[derive(Debug, Default)]
pub struct Stats {

}

impl Stats {

}

//--------------------
// Enums
//--------------------
#[derive(Debug, Default)]
pub enum PokemonType {
    #[default] None = -1,
    Normal,
    Fighting,
    Flying,
    Poison,
    Ground,
    Rock,
    Bug,
    Ghost,
    Steel,
    Fire,
    Water,
    Grass,
    Electric,
    Psychic,
    Ice,
    Dragon,
    Dark,
    Fairy,
    Stellar,
}

impl From<i32> for PokemonType {
    fn from(i: i32) -> Self {
        match i {
            0 => PokemonType::Normal,
            1 => PokemonType::Fighting,
            2 => PokemonType::Flying,
            3 => PokemonType::Poison,
            4 => PokemonType::Ground,
            5 => PokemonType::Rock,
            6 => PokemonType::Bug,
            7 => PokemonType::Ghost,
            8 => PokemonType::Steel,
            9 => PokemonType::Fire,
            10 => PokemonType::Water,
            11 => PokemonType::Grass,
            12 => PokemonType::Electric,
            13 => PokemonType::Psychic,
            14 => PokemonType::Ice,
            15 => PokemonType::Dragon,
            16 => PokemonType::Dark,
            17 => PokemonType::Fairy,
            18 => PokemonType::Stellar,
            _ => PokemonType::None,
        }
    }
}