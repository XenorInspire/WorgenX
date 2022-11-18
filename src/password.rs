// External crates
// use rand::Rng;

// Internal crates
use crate::system;

// This struct refers to a random password structure
struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
}

// This is the main function of the random password generation module
pub fn main_passwd_generation() {
    let password_config = allocate_passwd_config();
    println!("How many passwords do you want to generate ?");
    let number_of_passwords = system::get_user_choice_int();
    let passwords = generate_random_passwords(&password_config, number_of_passwords);
}

// This function is charged to allocate the password config structure
fn allocate_passwd_config() -> PasswordConfig {
    let mut password_config = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
    };
    let mut choice;

    println!("\nChoose what your password is composed of :");
    println!("Uppercase letters (A-Z) ? (y/n)");
    choice = system::get_user_choice();

    match &*choice {
        "y" => password_config.uppercase = true,
        _ => (),
    }

    println!("Lowercase letters (a-z) ? (y/n)");
    choice = system::get_user_choice();

    match &*choice {
        "y" => password_config.lowercase = true,
        _ => (),
    }

    println!("Numbers (0-9) ? (y/n)");
    choice = system::get_user_choice();

    match &*choice {
        "y" => password_config.numbers = true,
        _ => (),
    }

    println!("Special characters ? (y/n)");
    choice = system::get_user_choice();

    match &*choice {
        "y" => password_config.special_characters = true,
        _ => (),
    }

    println!("How long do you want your password to be ?");
    password_config.length = system::get_user_choice_int();

    return password_config;
}

// This function is charged to generate an array of random passwords
fn generate_random_passwords(password_config: &PasswordConfig, number_of_passwords: u64) -> Vec<String> {
    let mut passwords: Vec<String> = Vec::new();
    let mut password: String = String::new();

    for _ in 0..number_of_passwords {
        for _ in 0..password_config.length {
            // password.push(generate_random_char(&password_config));
        }
        passwords.push(password);
        password = String::new();
    }

    return passwords;
}
