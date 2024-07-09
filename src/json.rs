// Internal crates
use crate::password::PasswordConfig;

// External crates
use serde_json::json;

/// This function is charged to return a JSON String of the password config structure.
/// The output can be used to interact with other programs.
///
/// # Arguments
///
/// * `password_config` - The password config structure.
/// * `passwords` - The vector of passwords.
///
/// # Returns
///
/// A JSON String of the password config structure.
///
pub fn password_config_to_json(
    password_config: &PasswordConfig,
    passwords: &Vec<String>,
) -> String {
    json!({
        "number_of_passwords": password_config.number_of_passwords,
        "password_length": password_config.length,
        "uppercase": password_config.uppercase,
        "lowercase": password_config.lowercase,
        "numbers": password_config.numbers,
        "special_characters": password_config.special_characters,
        "passwords": passwords
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_password_config_to_json() {
        let password_config: PasswordConfig = PasswordConfig {
            numbers: true,
            special_characters: true,
            uppercase: true,
            lowercase: true,
            length: 10,
            number_of_passwords: 1,
        };
        let passwords: Vec<String> = vec!["password".to_string()];
        let json_output: String = password_config_to_json(&password_config, &passwords);

        let json_from_str: Value = serde_json::from_str(&json_output).unwrap();
        let json_expected_object: Value = json!({
            "number_of_passwords": 1,
            "password_length": 10,
            "uppercase": true,
            "lowercase": true,
            "numbers": true,
            "special_characters": true,
            "passwords": ["password"]
        });

        assert_eq!(json_from_str, json_expected_object);
    }
}
