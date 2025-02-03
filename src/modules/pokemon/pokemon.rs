use crate::pokemon::data::*;

// Handle generating a new pokemon
pub fn generate_new_pokemon(
    info: PokemonBase
) -> Pokemon {
    // Initialization
    let mut pokemon = Pokemon::initialize(info);
    pokemon.level = 5;
    pokemon.nature = PokemonNature::random();
    pokemon.iv = Stats::random_iv();

    // Stat Generation
    pokemon.stats.hp = generate_stat(true, pokemon.base_stats.hp,pokemon.ev.hp, pokemon.iv.hp, 5, &pokemon.nature, None);
    pokemon.current_hp = pokemon.stats.hp;
    
    pokemon.stats.attack = generate_stat(false, pokemon.base_stats.attack, pokemon.ev.attack, pokemon.iv.attack, 5, &pokemon.nature, Some(PokemonStat::Attack));
    pokemon.stats.defense = generate_stat(false, pokemon.base_stats.defense, pokemon.ev.defense, pokemon.iv.defense, 5, &pokemon.nature, Some(PokemonStat::Defense));
    pokemon.stats.special_attack = generate_stat(false, pokemon.base_stats.special_attack, pokemon.ev.special_attack, pokemon.iv.special_attack, 5, &pokemon.nature, Some(PokemonStat::SpecialAttack));
    pokemon.stats.special_defense = generate_stat(false, pokemon.base_stats.special_defense, pokemon.ev.special_defense, pokemon.iv.special_defense, 5, &pokemon.nature, Some(PokemonStat::SpecialDefense));
    pokemon.stats.speed = generate_stat(false, pokemon.base_stats.speed, pokemon.ev.speed, pokemon.iv.speed, 5, &pokemon.nature, Some(PokemonStat::Speed));

    pokemon
}

// Handle generating a new stat
fn generate_stat(
    is_hp: bool,
    base: u32,
    ev: u32,
    iv: u32,
    level: u32,
    nature: &PokemonNature,
    stat: Option<PokemonStat>
) -> u32 {
    let mut formula = (ev as f32 / 4.0).floor();
    formula = (2.0 * (base as f32) + (iv as f32) + formula) * (level as f32);
    formula = (formula / 100.0).floor();

    if is_hp {
        formula = formula + level as f32 + 10.0;
        formula as u32
    } else {
        let mult = nature.get_multiplier(stat.unwrap());

        formula = (formula + 5.0) * mult;
        formula.floor() as u32
    }
}