// Internal crates
use crate::dict;
use crate::system;

// External crates
use rand::{seq::SliceRandom, Rng};

/// This struct built from the user's choices will be used to generate the random password
/// 
struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
}

// This is the main function of the random password generation module
pub fn main_passwd_generation() {
    let mut again = String::from("y");

    while again.eq("y") {
        let password_config = allocate_passwd_config_gui();

        println!("How many passwords do you want to generate ?");
        let number_of_passwords = system::get_user_choice_int();

        println!("Do you want to save the passwords in a file ? (y/n)");
        let choice = system::get_user_choice_yn();
        let passwords = generate_random_passwords(&password_config, number_of_passwords);
        if choice.eq("y") {
            system::save_passwords_into_a_file(&passwords)
        };

        println!("You can find your password(s) below :");
        for password in passwords {
            println!("{}", password);
        }

        println!("\nDo you want to generate another password(s) ? (y/n)");
        again = system::get_user_choice_yn();
    }
}

/// This function is charged to allocate the password config structure from the user's choices in the GUI
///
/// # Example
/// ```
/// let password_config = allocate_passwd_config_gui();
/// ```
/// # Returns
/// 
/// The password config structure named PasswordConfig
/// 
fn allocate_passwd_config_gui() -> PasswordConfig {
    let mut password_config = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
    };
    let mut choice;
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your password is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.uppercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.lowercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Numbers (0-9) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.numbers = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Special characters ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                password_config.special_characters = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        if !is_option_chosen {
            println!("You must choose at least one option !");
        }
    }

    println!("How long do you want your password to be ?");
    password_config.length = system::get_user_choice_int();

    password_config
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
fn generate_random_passwords(
    password_config: &PasswordConfig,
    number_of_passwords: u64,
) -> Vec<String> {
    let mut passwords: Vec<String> = Vec::new();
    let password_content: Vec<u8> = create_passwd_content(password_config);

    for _ in 0..number_of_passwords {
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
