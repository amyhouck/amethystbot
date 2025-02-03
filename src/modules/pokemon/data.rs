use chrono::NaiveDate;
use rand::Rng;
use serde::ser::Impossible;

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
    pub nature: PokemonNature,

    // Stats
    pub level: u32,
    pub current_hp: u32,
    pub stats: Stats,
    pub base_stats: Stats,
    pub iv: Stats,
    pub ev: Stats
}

impl Pokemon {
    pub fn initialize(base: PokemonBase) -> Self {
        let base_stats = Stats {
            hp: base.base_hp,
            attack: base.base_attack,
            defense: base.base_defense,
            special_attack: base.base_special_attack,
            special_defense: base.base_special_defense,
            speed: base.base_speed
        };

        Self {
            id: base.id,
            name: base.name,
            form: base.form,
            type_1: base.type_1,
            type_2: base.type_2,
            base_stats,

            ..Default::default()
        }
    }
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

    // Base Stats
    pub base_hp: u32,
    pub base_attack: u32,
    pub base_defense: u32,
    pub base_special_attack: u32,
    pub base_special_defense: u32,
    pub base_speed: u32
}

// Stats Structure
#[derive(Debug, Default)]
pub struct Stats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub special_attack: u32,
    pub special_defense: u32,
    pub speed: u32
}

impl Stats {
    pub fn random_iv() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            hp: rng.gen_range(0..=31),
            attack: rng.gen_range(0..=31),
            defense: rng.gen_range(0..=31),
            special_attack: rng.gen_range(0..=31),
            special_defense: rng.gen_range(0..=31),
            speed: rng.gen_range(0..=31),
        }
    }
}

// Stat enum to help determine nature multiplier in generation
pub enum PokemonStat {
    // HP
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed
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

#[derive(Clone, Copy, Debug, Default)]
pub enum PokemonNature {
    #[default] Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky
}

impl From<u32> for PokemonNature {
    fn from(i: u32) -> Self {
        match i {
            0 => Self::Hardy,
            1 => Self::Lonely,
            2 => Self::Brave,
            3 => Self::Adamant,
            4 => Self::Naughty,
            5 => Self::Bold,
            6 => Self::Docile,
            7 => Self::Relaxed,
            8 => Self::Impish,
            9 => Self::Lax,
            10 => Self::Timid,
            11 => Self::Hasty,
            12 => Self::Serious,
            13 => Self::Jolly,
            14 => Self::Naive,
            15 => Self::Modest,
            16 => Self::Mild,
            17 => Self::Quiet,
            18 => Self::Bashful,
            19 => Self::Rash,
            20 => Self::Calm,
            21 => Self::Gentle,
            22 => Self::Sassy,
            23 => Self::Careful,
            24 => Self::Quirky,
            _ => Self::Hardy
        }
    }
}

impl PokemonNature {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen_range(0..=24);

        PokemonNature::from(num)
    }

    pub fn get_multiplier(self, stat: PokemonStat) -> f32 {
        match stat {
            PokemonStat::Attack => {
                match self {
                    Self::Lonely | Self::Brave | Self::Adamant | Self::Naughty => 1.1,
                    Self::Bold | Self::Timid | Self::Modest | Self::Calm => 0.9,
                    _ => 1.0
                }
            },

            PokemonStat::Defense => {
                match self {
                    Self::Bold | Self::Relaxed | Self::Impish | Self::Lax => 1.1,
                    Self::Lonely | Self::Hasty | Self::Mild | Self::Gentle => 0.9,
                    _ => 1.0
                }
            },

            PokemonStat::Speed => {
                match self {
                    Self::Timid | Self::Hasty | Self::Jolly | Self::Naive => 1.1,
                    Self::Brave | Self::Relaxed | Self::Quiet | Self::Sassy => 0.9,
                    _ => 1.0
                }
            },

            PokemonStat::SpecialAttack => {
                match self {
                    Self::Modest | Self::Mild | Self::Quiet | Self::Rash => 1.1,
                    Self::Adamant | Self::Impish | Self::Jolly | Self::Careful => 0.9,
                    _ => 1.0
                }
            },

            PokemonStat::SpecialDefense => {
                match self {
                    Self::Calm | Self::Gentle | Self::Sassy | Self::Careful => 1.1,
                    Self::Naughty | Self::Lax | Self::Naive | Self::Rash => 0.9,
                    _ => 1.0
                }
            }
        }
    }
}