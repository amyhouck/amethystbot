use crate::pokemon::data::{Pokemon, Move, PokemonType, PokemonStats, PokemonNature};
use rand::{thread_rng, Rng};

pub enum IvEvOptions {
    Normal,
    SetIV(PokemonStats),
    SetEV(PokemonStats),
    SetBoth(PokemonStats, PokemonStats)
}

// -- Primary generator function
pub async fn generate_pokemon(
    database: &sqlx::SqlitePool,
    pokemon: String,
    level: i64,
    iv_ev_options: IvEvOptions
) -> Pokemon {
    // Grab base data and initialize things
    let mut new_pokemon: Pokemon = Pokemon::new();

    let base_data = sqlx::query!("SELECT * FROM pokemon WHERE name = ?", pokemon)
        .fetch_one(database)
        .await
        .unwrap();

    new_pokemon.name = base_data.name;
    new_pokemon.id = base_data.id;
    new_pokemon.level = level;
    new_pokemon.poke_type = (base_data.type_1.unwrap().into(), base_data.type_2.unwrap().into());
    new_pokemon.current_hp = base_data.base_hp.unwrap();
    new_pokemon.base_stats = PokemonStats {
        hp: base_data.base_hp.unwrap(),
        attack: base_data.base_attack.unwrap(),
        defense: base_data.base_defense.unwrap(),
        special_attack: base_data.base_special_attack.unwrap(),
        special_defense: base_data.base_special_defense.unwrap(),
        speed: base_data.base_speed.unwrap(),
    };

    // Set IVs and EVs
    match iv_ev_options {
        _ => {
            let mut rng = thread_rng();
            new_pokemon.ivs = PokemonStats {
                hp: rng.gen_range(0..=31),
                attack: rng.gen_range(0..=31),
                defense: rng.gen_range(0..=31),
                special_attack: rng.gen_range(0..=31),
                special_defense: rng.gen_range(0..=31),
                speed: rng.gen_range(0..=31)
            };
        }
    }

    // Generate nature and stats
    new_pokemon.nature = {
        let mut rng = thread_rng();
        rng.gen_range(0..=24).into()
    };
    new_pokemon.stats.hp = generate_stat(new_pokemon.base_stats.hp as f64, new_pokemon.ivs.hp as f64, new_pokemon.evs.hp as f64, level as f64, None);
    new_pokemon.stats.attack = generate_stat(new_pokemon.base_stats.attack as f64, new_pokemon.ivs.attack as f64, new_pokemon.evs.attack as f64, level as f64, Some(new_pokemon.nature.get_multiplier("attack")));
    new_pokemon.stats.defense = generate_stat(new_pokemon.base_stats.defense as f64, new_pokemon.ivs.defense as f64, new_pokemon.evs.defense as f64, level as f64, Some(new_pokemon.nature.get_multiplier("defense")));
    new_pokemon.stats.special_attack = generate_stat(new_pokemon.base_stats.special_attack as f64, new_pokemon.ivs.special_attack as f64, new_pokemon.evs.special_attack as f64, level as f64, Some(new_pokemon.nature.get_multiplier("special_attack")));
    new_pokemon.stats.special_defense = generate_stat(new_pokemon.base_stats.special_defense as f64, new_pokemon.ivs.special_defense as f64, new_pokemon.evs.special_defense as f64, level as f64, Some(new_pokemon.nature.get_multiplier("special_defense")));
    new_pokemon.stats.speed = generate_stat(new_pokemon.base_stats.speed as f64, new_pokemon.ivs.speed as f64, new_pokemon.evs.speed as f64, level as f64, Some(new_pokemon.nature.get_multiplier("speed")));

    new_pokemon
}

// -- Function: apply stat formulae
fn generate_stat(base: f64, iv: f64, ev: f64, level: f64, nat_mult: Option<f64>) -> i64 {
    let stat = match nat_mult {
        Some(nat) => { // -- Return other stat
            (((((2.0 * base + iv + (ev / 4.0)) * level)
                / 100.0)
                .floor()
                + 5.0)
                * nat).floor()
        },
        None => { // -- Return HP Value
            (((2.0 * base + iv + (ev / 4.0).floor())
                * level)
                / 100.0).floor()
                + level
                + 10.0
        }
    };

    stat as i64
}