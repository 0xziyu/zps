use eyre::{Result, eyre};
use rand::{rng, seq::IndexedRandom};
use tracing::info;

const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:',.<>/?";

// This function will be called by `add` and `generate` commands
pub fn generate_password_internal(length: usize, no_symbols: bool) -> String {
    let mut rng = rng();
    let mut char_pool: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();

    if !no_symbols {
        char_pool.extend(SYMBOLS.iter().map(|&s| s as char));
    }

    if char_pool.is_empty() {
        return String::new();
    }

    let password = (0..length)
        .map(|_| *char_pool.choose(&mut rng).unwrap())
        .collect();
    info!("Generated password: {}", password);
    password
}

pub fn handle_pass_generate(length: usize, no_symbols: bool) -> Result<()> {
    if length == 0 {
        return Err(eyre!("Password length cannot be zero."));
    }
    generate_password_internal(length, no_symbols);
    Ok(())
}
