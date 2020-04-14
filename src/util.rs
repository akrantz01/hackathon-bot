use log::error;
use std::{env, iter, process::exit};

use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};
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
    pub static ref DISCORD_TOKEN: String = parse_from_environment::<String>("DISCORD_TOKEN");
    pub static ref REDIS_URL: String = parse_from_environment::<String>("REDIS_URL");
    pub static ref TABLES_CATEGORY_ID: u64 = parse_from_environment::<u64>("TABLES_CATEGORY_ID");
    pub static ref REPORTS_CHANNEL_ID: u64 = parse_from_environment::<u64>("REPORTS_CHANNEL_ID");
    pub static ref EVERYONE_ROLE_ID: u64 = parse_from_environment::<u64>("EVERYONE_ROLE_ID");
    pub static ref TEAMLESS_ROLE_ID: u64 = parse_from_environment::<u64>("TEAMLESS_ROLE_ID");
    pub static ref BOT_ROLE_ID: u64 = parse_from_environment::<u64>("BOT_ROLE_ID");
    pub static ref MENTOR_ROLE_ID: u64 = parse_from_environment::<u64>("MENTOR_ROLE_ID");
    pub static ref MANAGER_ROLE_ID: u64 = parse_from_environment::<u64>("MANAGER_ROLE_ID");
}

// Parse some type from a given environment variable
fn parse_from_environment<T: std::str::FromStr>(var: &'static str) -> T {
    let raw = match env::var(var) {
        Ok(raw) => raw,
        Err(_) => fail(&format!("Variable '{}' is nonexistent! Exiting...", var)),
    };

    match raw.parse::<T>() {
        Ok(parsed) => parsed,
        Err(_) => fail(&format!(
            "Variable '{}' must be of type '{}'! Exiting...",
            var,
            std::any::type_name::<T>()
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

/// Get a random string
pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect()
}
