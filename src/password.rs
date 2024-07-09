// Internal crates
use crate::dict;

// External crates
use rand::{rngs::OsRng, seq::SliceRandom, Rng};

/// This struct built from the user's choices will be used to generate the random password.
///
#[derive(Debug)]
pub struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u32,
    pub number_of_passwords: u64,
}

/// This function is charged to create the content of the password.
/// It returns a vector of u8 containing the characters that will be used to generate the password.
///
/// # Arguments
///
/// * `password_config` - The struct containing the user's choices.
///
/// # Returns
///
/// The vector of u8 containing the characters that will be used to generate the password.
/// All the characters are shuffled in a random order.
///
fn create_passwd_content(password_config: &PasswordConfig) -> Vec<u8> {
    let mut password_content: Vec<u8> = Vec::new();

    if password_config.uppercase {
        password_content.extend(shuffle_dict(dict::UPPERCASE));
    }

    if password_config.lowercase {
        password_content.extend(shuffle_dict(dict::LOWERCASE));
    }

    if password_config.numbers {
        password_content.extend(shuffle_dict(dict::NUMBERS));
    }

    if password_config.special_characters {
        password_content.extend(shuffle_dict(dict::SPECIAL_CHARACTERS));
    }

    let mut rng: OsRng = OsRng;
    password_content.shuffle(&mut rng);
    password_content
}

/// This function is charged to shuffle a vector of u8 from the dict module.
///
/// # Arguments
///
/// * `dict` - The dictionary of u8 to shuffle.
///
/// # Returns
///
/// The shuffled vector of u8 sent in parameter.
///
fn shuffle_dict(dict: &[u8]) -> Vec<u8> {
    let mut shuffled_dict: Vec<u8> = dict.to_vec();
    let mut rng: OsRng = OsRng;
    shuffled_dict.shuffle(&mut rng);
    shuffled_dict
}

/// This function is charged to generate an array of random passwords.
///
/// # Arguments
///
/// * `password_config` - The password config structure.
/// * `number_of_passwords` - The number of passwords to generate.
///
/// # Returns
///
/// A vector of String containing the random passwords.
///
pub fn generate_random_passwords(password_config: &PasswordConfig) -> Vec<String> {
    let mut passwords: Vec<String> = Vec::new();
    let password_content: Vec<u8> = create_passwd_content(password_config);

    for _ in 0..password_config.number_of_passwords {
        let mut rng: OsRng = OsRng;
        let mut password: String = String::new();
        for _ in 0..password_config.length {
            let idx: usize = rng.gen_range(0..password_content.len());
            password.push(password_content[idx] as char);
        }
        passwords.push(password);
    }

    passwords
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_passwd_content() {
        let password_config: PasswordConfig = PasswordConfig {
            numbers: true,
            special_characters: true,
            uppercase: true,
            lowercase: true,
            length: 10,
            number_of_passwords: 1,
        };
        let password_content: Vec<u8> = create_passwd_content(&password_config);
        
        assert_eq!(password_content.len(), 91);
    }

    #[test]
    fn test_shuffle_dict() {
        let dict: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let shuffled_dict: Vec<u8> = shuffle_dict(&dict);

        assert_eq!(dict.len(), shuffled_dict.len());
    }

    #[test]
    fn test_generate_random_passwords() {
        let password_config: PasswordConfig = PasswordConfig {
            numbers: true,
            special_characters: true,
            uppercase: true,
            lowercase: true,
            length: 10,
            number_of_passwords: 1,
        };
        let passwords: Vec<String> = generate_random_passwords(&password_config);

        assert_eq!(passwords.len(), 1);
        assert_eq!(passwords[0].len(), 10);
    }
}
