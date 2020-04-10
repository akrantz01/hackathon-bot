use log::error;
use std::{env, process::exit};

use rand::seq::SliceRandom;
use serenity::utils::Colour;

const COLORS: [Colour; 25] = [
    Colour::BLITZ_BLUE,
    Colour::BLURPLE,
    Colour::DARK_BLUE,
    Colour::DARK_GOLD,
    Colour::DARK_GREY,
    Colour::DARK_MAGENTA,
    Colour::DARK_ORANGE,
    Colour::DARK_PURPLE,
    Colour::DARK_TEAL,
    Colour::DARKER_GREY,
    Colour::FABLED_PINK,
    Colour::FADED_PURPLE,
    Colour::FOOYOO,
    Colour::GOLD,
    Colour::KERBAL,
    Colour::LIGHT_GREY,
    Colour::LIGHTER_GREY,
    Colour::MAGENTA,
    Colour::MEIBE_PINK,
    Colour::ORANGE,
    Colour::PURPLE,
    Colour::RED,
    Colour::ROHRKATZE_BLUE,
    Colour::ROSEWATER,
    Colour::TEAL,
];

// Retrieve ids from environment
lazy_static! {
    pub static ref TABLES_CATEGORY_ID: u64 = parse_id_from_environment("TABLES_CATEGORY_ID");
    pub static ref EVERYONE_ROLE_ID: u64 = parse_id_from_environment("EVERYONE_ROLE_ID");
    pub static ref TEAMLESS_ROLE_ID: u64 = parse_id_from_environment("TEAMLESS_ROLE_ID");
    pub static ref MENTOR_ROLE_ID: u64 = parse_id_from_environment("MENTOR_ROLE_ID");
    pub static ref MANAGER_ROLE_ID: u64 = parse_id_from_environment("MANAGER_ROLE_ID");
}

// Parse an id (u64) from a given environment variable
fn parse_id_from_environment(var: &'static str) -> u64 {
    let raw_id = match env::var(var) {
        Ok(id) => id,
        Err(_) => fail(&format!("Variable '{}' is nonexistent! Exiting...", var)),
    };

    match raw_id.parse::<u64>() {
        Ok(id) => id,
        Err(_) => fail(&format!(
            "Variable '{}' must be an unsigned 64-bit integer! Exiting...",
            var
        )),
    }
}

/// Print an error and exit with error code
pub fn fail(prompt: &'_ str) -> ! {
    error!("{}", prompt);
    exit(1);
}

/// Get a random color
pub fn random_color() -> &'static Colour {
    COLORS.choose(&mut rand::thread_rng()).unwrap()
}
