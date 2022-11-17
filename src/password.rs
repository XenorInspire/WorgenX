// Extern crates
use std::io;

use crate::system;

struct PasswordConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
    pub content: String,
}

fn allocatePasswdConfig() -> PasswordConfig {
    let mut password_config = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
        content: String::from(""),
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
