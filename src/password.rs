// Internal crates
use crate::dict;

// External crates
use rand::{seq::SliceRandom, Rng};

/// This struct built from the user's choices will be used to generate the random password
///
pub struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
    pub number_of_passwords: u64,
    pub output_file: String,
    pub json: bool,
}

/// This function is charged to create the content of the password
/// It returns a vector of u8 containing the characters that will be used to generate the password
///
/// # Example
/// ```
/// let password_content = create_passwd_content(&password_config);
/// ```
///
/// # Returns
///
/// The vector of u8 containing the characters that will be used to generate the password
///
fn create_passwd_content(password_config: &PasswordConfig) -> Vec<u8> {
    let mut password_families: Vec<Vec<u8>> = Vec::new();

    if password_config.uppercase {
        password_families.push(shuffle_dict(&dict::UPPERCASE.to_vec()));
    }

    if password_config.lowercase {
        password_families.push(shuffle_dict(&dict::LOWERCASE.to_vec()));
    }

    if password_config.numbers {
        password_families.push(shuffle_dict(&dict::NUMBERS.to_vec()));
    }

    if password_config.special_characters {
        password_families.push(shuffle_dict(&dict::SPECIAL_CHARACTERS.to_vec()));
    }

    // Generate the random indexes of the password families in a random order
    let mut rng = rand::thread_rng();
    let mut password_families_indexes: Vec<usize> = (0..password_families.len()).collect();
    password_families_indexes.shuffle(&mut rng);

    // Generate the final array of u8 containing the password content
    let mut password_content: Vec<u8> = Vec::new();
    for i in password_families_indexes {
        password_content.extend(password_families[i].to_vec());
    }
    let mut rng = rand::thread_rng();
    password_content.shuffle(&mut rng);
    password_content
}

/// This function is charged to shuffle a vector of u8 from the dict module
///
/// # Example
/// ```
/// let mut shuffled_uppercase = shuffle_dict(&dict::UPPERCASE.to_vec());
/// let mut shuffled_lowercase = shuffle_dict(&dict::LOWERCASE.to_vec());
/// let mut shuffled_numbers = shuffle_dict(&dict::NUMBERS.to_vec());
/// let mut shuffled_special_characters = shuffle_dict(&dict::SPECIAL_CHARACTERS.to_vec());
/// ```
///
/// # Returns
///
/// The shuffled vector of u8 passed in parameter
///
fn shuffle_dict(dict: &Vec<u8>) -> Vec<u8> {
    let mut shuffled_dict = dict.to_vec();
    let mut rng = rand::thread_rng();
    shuffled_dict.shuffle(&mut rng);
    shuffled_dict
}

/// This function is charged to generate an array of random passwords
///
/// # Arguments
///
/// * `password_config` - The password config structure
/// * `number_of_passwords` - The number of passwords to generate
///
/// # Example
/// ```
/// let passwords = generate_random_passwords(&password_config, number_of_passwords);
/// ```
///
pub fn generate_random_passwords(password_config: &PasswordConfig) -> Vec<String> {
    let mut passwords: Vec<String> = Vec::new();
    let password_content: Vec<u8> = create_passwd_content(password_config);

    for _ in 0..password_config.number_of_passwords {
        let mut rng = rand::thread_rng();
        let mut password: String = String::new();
        for _ in 0..password_config.length {
            let idx = rng.gen_range(0..password_content.len());
            password.push(password_content[idx] as char);
        }
        passwords.push(password);
    }

    passwords
}
