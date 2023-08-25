// Internal crates
use crate::password::PasswordConfig;

// External crates
use serde_json::json;

/// This function is charged to return a JSON String of the password config structure
///
/// # Arguments
///
/// * `password_config` - The password config structure
///
/// # Returns
///
/// A JSON String of the password config structure
///
pub fn password_config_to_json(
    password_config: &PasswordConfig,
    passwords: &Vec<String>,
) -> String {
    let json = json!({
        "number_of_passwords": password_config.number_of_passwords,
        "password_length": password_config.length,
        "uppercase": password_config.uppercase,
        "lowercase": password_config.lowercase,
        "numbers": password_config.numbers,
        "special_characters": password_config.special_characters,
        // Display the passwords in a JSON array
        "passwords": passwords
    });

    json.to_string()
}
