// Internal crates
use crate::password::{self, PasswordConfig};
use crate::system;

pub fn run(){
   loop {
        print_menu();
        let choice = system::get_user_choice();
        match &*choice {
            "0" => break,
            // "1" => generate_wordlist(),
            "2" => main_passwd_generation(),
            // "3" => benchmark_cpu(),
            _ => (),
        }
    }
    println!("Bye!"); 
}

/// This function is charged to display the header menu
///
/// # Example
/// ```
/// display_title();
/// ```
/// 
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

/// This function is charged to display the menu
///
/// # Example
/// ```
/// print_menu();
/// ```
/// 
fn print_menu() {
    display_title();
    println!("\n   WorgenX by Xen0rInspire \n");
    display_title();

    print!("\n\n");
    println!("1 : Create wordlist(s)");
    println!("2 : Generate random password(s)");
    println!("3 : Benchmark CPU");
    println!("0 : Exit WorgenX");
}

// This is the main function of the random password generation module
fn main_passwd_generation() {
    let mut again = String::from("y");

    while again.eq("y") {
        let password_config = allocate_passwd_config_gui();

        println!("How many passwords do you want to generate ?");
        let number_of_passwords = system::get_user_choice_int();

        println!("Do you want to save the passwords in a file ? (y/n)");
        let choice = system::get_user_choice_yn();
        let passwords = password::generate_random_passwords(&password_config, number_of_passwords);
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