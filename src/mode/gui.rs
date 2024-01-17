// Internal crates
use crate::{
    password::{self, PasswordConfig},
    system,
};

// External crates
use std::{env, fs::OpenOptions, sync::Arc, sync::Mutex};

#[cfg(target_family = "unix")]
use system::unix as target;

#[cfg(target_family = "windows")]
use system::windows as target;

/// This function is charged to schedule in GUI mode the execution of the different features of the program
/// according to the user's choices
///
pub fn run() {
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
fn display_title() {
    for _ in 0..30 {
        print!("#");
    }
    println!();
}

/// This function is charged to display the menu
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
        let passwords = password::generate_random_passwords(&password_config);

        println!("You can find your password(s) below :");
        for password in &passwords {
            println!("{}", password);
        }

        println!("\nDo you want to save the passwords in a file ? (y/n)");
        let choice = system::get_user_choice_yn();
        if choice.eq("y") {
            password_saving_procedure(&passwords)
        }

        println!("\nDo you want to generate another password(s) ? (y/n)");
        again = system::get_user_choice_yn();
    }

    println!("\n");
}

/// This function is charged to allocate the password config structure from the user's choices in the GUI
///
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
        number_of_passwords: 0,
    };
    let mut choice;
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your password is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        choice = system::get_user_choice_yn();
        if &*choice == "y" {
            password_config.uppercase = true;
            is_option_chosen = true;
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        choice = system::get_user_choice_yn();
        if &*choice == "y" {
            password_config.lowercase = true;
            is_option_chosen = true;
        }

        println!("Numbers (0-9) ? (y/n)");
        choice = system::get_user_choice_yn();
        if &*choice == "y" {
            password_config.numbers = true;
            is_option_chosen = true;
        }

        println!("Special characters ? (y/n)");
        choice = system::get_user_choice_yn();
        if &*choice == "y" {
            password_config.special_characters = true;
            is_option_chosen = true;
        }

        if !is_option_chosen {
            println!("You must choose at least one option !");
        }
    }

    println!("How long do you want your password to be ?");
    password_config.length = system::get_user_choice_int();

    println!("How many passwords do you want to generate ?");
    password_config.number_of_passwords = system::get_user_choice_int();

    password_config
}

/// This function is charged to save the random password in a file with \n as separator in GUI mode
///
/// # Arguments
///
/// * `passwords` - A vector of String that holds the passwords to save
///
pub fn password_saving_procedure(passwords: &[String]) {
    println!("Please enter the file name");
    let mut filename = system::get_user_choice();
    let mut result = system::is_valid_path(filename.clone());
    while result.is_err() {
        println!("{}", result.unwrap_err());
        println!("Please enter a new file name:");
        filename = system::get_user_choice();
        result = system::is_valid_path(filename.clone());
    }

    let filename = match env::var(target::HOME_ENV_VAR) {
        Ok(home_path) => {
            let parent_folder = format!("{}{}", home_path, target::PASSWORDS_FOLDER);
            let parent_folder_created = match system::create_folder_if_not_exists(&parent_folder) {
                Ok(_) => format!("{}{}", home_path, target::PASSWORDS_FOLDER),
                Err(e) => {
                    println!("{}", e);
                    println!("Unable to create the folder, the passwords will be saved in the current directory");
                    format!("./{}", filename)
                }
            };
            format!("{}{}", parent_folder_created, filename)
        }
        Err(_) => {
            println!("Unable to get the home directory, the passwords will be saved in the current directory\n");
            format!("./{}", filename)
        }
    };

    let file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(filename.clone())
    {
        Ok(file) => file,
        Err(_) => {
            println!(
                "Unable to open the file, the passwords will be saved in the current directory\n"
            );
            match OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(filename.clone())
            {
                Ok(file) => file,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            }
        }
    };
    let shared_file = Arc::new(Mutex::new(file));

    while let Err(e) = system::save_passwd_to_file(shared_file.clone(), passwords.join("\n")) {
        println!("\n{}", e);
        println!("Do you want to try again ? (y/n)");
        let choice = system::get_user_choice_yn();
        if choice.eq("n") {
            return;
        }
    }

    println!("The passwords have been saved in {}", filename);
}
