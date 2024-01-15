// -- Starter Pokemon
#[derive(Debug, poise::ChoiceParameter)]
pub enum StarterPokemon {
    Bulbasaur,
    Charmander,
    Squirtle,
}

impl StarterPokemon {
    fn to_string(&self) -> String {
        match self {
            StarterPokemon::Bulbasaur => String::from("Bulbasaur"),
            StarterPokemon::Charmander => String::from("Charmander"),
            StarterPokemon::Squirtle => String::from("Squirtle"),
        }
    }
}

// -- Pokemon Data
#[derive(Debug)]
pub struct Pokemon {
    pub name: String,
    pub id: i64,
    pub level: i64,
    pub poke_type: (PokemonType, PokemonType),
    pub status: PokemonStatus,
    pub nature: PokemonNature,
    pub current_hp: i64,
    pub base_stats: PokemonStats,
    pub ivs: PokemonStats,
    pub evs: PokemonStats,
    pub stats: PokemonStats,
    pub moves: Vec<Move>,
}

impl Pokemon {
    pub fn new() -> Self {
        return Pokemon {
            name: "Missingno.".to_string(),
            id: 0,
            level: 0,
            poke_type: (PokemonType::from(0), PokemonType::from(0)),
            status: PokemonStatus::Normal,
            nature: PokemonNature::Hardy,
            current_hp: 0,
            base_stats: PokemonStats::new(),
            ivs: PokemonStats::new(),
            evs: PokemonStats::new(),
            stats: PokemonStats::new(),
            moves: Vec::new(),
        };
    }
}

#[derive(Debug)]
pub struct PokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub special_attack: i64,
    pub special_defense: i64,
    pub speed: i64
}

impl PokemonStats {
    pub fn new() -> Self {
        PokemonStats {
            hp: 0,
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0
        }
    }
}

// -- Move Data
#[derive(Debug)]
pub struct Move {
    pub name: String,
    pub id: u32,
    pub move_type: i32,
    pub effect_type: u32,
    pub points: (u32, u32),
    pub power: u32,
    pub accuracy: u32,
    pub effect: u32,
}

// -- Nature Data
#[derive(Debug)]
pub enum PokemonNature {
    Hardy = 0,
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

impl From<i64> for PokemonNature {
    fn from(nature_int: i64) -> Self {
        match nature_int {
            0 => PokemonNature::Hardy,
            1 => PokemonNature::Lonely,
            2 => PokemonNature::Brave,
            3 => PokemonNature::Adamant,
            4 => PokemonNature::Naughty,
            5 => PokemonNature::Bold,
            6 => PokemonNature::Docile,
            7 => PokemonNature::Relaxed,
            8 => PokemonNature::Impish,
            9 => PokemonNature::Lax,
            10 => PokemonNature::Timid,
            11 => PokemonNature::Hasty,
            12 => PokemonNature::Serious,
            13 => PokemonNature::Jolly,
            14 => PokemonNature::Naive,
            15 => PokemonNature::Modest,
            16 => PokemonNature::Mild,
            17 => PokemonNature::Quiet,
            18 => PokemonNature::Bashful,
            19 => PokemonNature::Rash,
            20 => PokemonNature::Calm,
            21 => PokemonNature::Gentle,
            22 => PokemonNature::Sassy,
            23 => PokemonNature::Careful,
            24 => PokemonNature::Quirky,
            _ => PokemonNature::Hardy
        }
    }
}

impl PokemonNature {
    // Get stat multiplier
    pub fn get_multiplier(&self, stat: &str) -> f64 {
        match stat {
            "attack" => {
                match self {
                    PokemonNature::Lonely | PokemonNature::Brave | PokemonNature::Adamant | PokemonNature::Naughty => 1.1,
                    PokemonNature::Bold | PokemonNature::Timid | PokemonNature::Modest | PokemonNature::Calm => 0.9,
                    _ => 1.0
                }
            },
            "defense" => {
                match self {
                    PokemonNature::Bold | PokemonNature::Relaxed | PokemonNature::Impish | PokemonNature::Lax => 1.1,
                    PokemonNature::Lonely | PokemonNature::Hasty | PokemonNature::Mild | PokemonNature::Gentle => 0.9,
                    _ => 1.0
                }
            },
            "speed" => {
                match self {
                    PokemonNature::Timid | PokemonNature::Hasty | PokemonNature::Jolly | PokemonNature::Naive => 1.1,
                    PokemonNature::Brave | PokemonNature::Relaxed | PokemonNature::Quiet | PokemonNature::Sassy => 0.9,
                    _ => 1.0
                }
            },
            "special_attack" => {
                match self {
                    PokemonNature::Modest | PokemonNature::Mild | PokemonNature::Quiet | PokemonNature::Rash => 1.1,
                    PokemonNature::Adamant | PokemonNature::Impish | PokemonNature::Jolly | PokemonNature::Careful => 0.9,
                    _ => 1.0
                }
            },
            "special_defense" => {
                match self {
                    PokemonNature::Calm | PokemonNature::Gentle | PokemonNature::Sassy | PokemonNature::Careful => 1.1,
                    PokemonNature::Naughty | PokemonNature::Lax | PokemonNature::Naive | PokemonNature::Rash => 0.9,
                    _ => 1.0
                }
            }
            _ => 0.0
        }
    }
}

// -- Type Data
#[derive(Debug)]
pub enum PokemonType {
    None = -1,
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
    Fairy
}

impl From<i64> for PokemonType {
    fn from(poketype: i64) -> Self {
        match poketype {
            -1 => PokemonType::None,
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
            _ => PokemonType::None
        }
    }
}

// -- Status Data
#[derive(Debug)]
pub enum PokemonStatus {
    Normal = 0,
    Burn,
    Freeze,
    Frostbite,
    Paralysis,
    Poison,
    BadPoison,
    Sleep,
    Drowsy,
}

impl From<i64> for PokemonStatus {
    fn from(status_int: i64) -> Self {
        match status_int {
            0 => PokemonStatus::Normal,
            1 => PokemonStatus::Burn,
            2 => PokemonStatus::Freeze,
            3 => PokemonStatus::Frostbite,
            4 => PokemonStatus::Paralysis,
            5 => PokemonStatus::Poison,
            6 => PokemonStatus::BadPoison,
            7 => PokemonStatus::Sleep,
            8 => PokemonStatus::Drowsy,
            _ => PokemonStatus::Normal
        }
    }
}