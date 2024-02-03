// Internal crates
use crate::{
    error::SystemError,
    password::{self, PasswordConfig},
    system,
    wordlist::{self, WordlistValues},
};

// External crates
use std::{
    env,
    fs::{File, OpenOptions},
    sync::Arc,
    sync::Mutex,
};

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
            "1" => main_wordlist_generation(),
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

/// This is the main function of the random password generation feature
///
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
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your password is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            password_config.uppercase = true;
            is_option_chosen = true;
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            password_config.lowercase = true;
            is_option_chosen = true;
        }

        println!("Numbers (0-9) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            password_config.numbers = true;
            is_option_chosen = true;
        }

        println!("Special characters ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
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

/// This function is charged to save the random password in a file
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
            // TODO: check if it's necessary to handle the error
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

/// This is the main function of the wordlist generation feature
///
fn main_wordlist_generation() {
    let mut again = String::from("y");

    while again.eq("y") {
        let wordlist_values = allocate_wordlist_config_gui();
        let wordlist_config = wordlist::build_wordlist_config(&wordlist_values);
        let nb_of_passwd = wordlist_config
            .dict
            .len()
            .pow(wordlist_config.mask_indexes.len() as u32) as u64;

        let mut wordlist_file = wordlist_saving_procedure();
        while wordlist_file.is_err() {
            println!("{}", wordlist_file.unwrap_err());
            println!("Do you want to try again ? (y/n)");
            if system::get_user_choice_yn().eq("n") {
                return;
            }
            wordlist_file = wordlist_saving_procedure();
        }

        // TODO: processing + threads management + progress bar

        println!("\nDo you want to generate another wordlist ? (y/n)");
        again = system::get_user_choice_yn();
    }

    println!("\n");
}

/// This function is charged to allocate the wordlist config structure from the user's choices
///
/// # Returns
///
/// The wordlist values structure named WordlistValues
///
fn allocate_wordlist_config_gui() -> WordlistValues {
    let mut wordlist_config = WordlistValues {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        mask: String::new(),
    };
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your wordlist is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            wordlist_config.uppercase = true;
            is_option_chosen = true;
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            wordlist_config.lowercase = true;
            is_option_chosen = true;
        }

        println!("Numbers (0-9) ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            wordlist_config.numbers = true;
            is_option_chosen = true;
        }

        println!("Special characters ? (y/n)");
        if system::get_user_choice_yn().eq("y") {
            wordlist_config.special_characters = true;
            is_option_chosen = true;
        }

        if !is_option_chosen {
            println!("You must choose at least one option !");
        }
    }

    println!("Enter the mask of the wordlist :");
    println!("For every character you want to be fixed, enter the character itself.");
    println!("For every character you want to be variable, enter a ?.");
    println!(
        "If you want to specify the character '?' in the mask as a fixed character, enter '\\?'"
    );

    let mut is_valid_mask = false;
    while !is_valid_mask {
        wordlist_config.mask = system::get_user_choice();
        if wordlist_config.mask.is_empty() {
            println!("The mask cannot be empty !");
            continue;
        } else if !wordlist_config.mask.contains('?') {
            println!("The mask must contain at least one '?' !");
            continue;
        } else {
            println!(
                "Do you want to validate the following mask : {} ? (y/n)",
                wordlist_config.mask
            );
            if system::get_user_choice_yn().eq("y") {
                is_valid_mask = true;
            }
        }
    }

    wordlist_config
}

/// This function is charged to save the wordlist in a file with \n as separator
///
/// # Returns
///
/// The file where the wordlist will be saved : Result<File, SystemError>
///
pub fn wordlist_saving_procedure() -> Result<File, SystemError> {
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
            let parent_folder = format!("{}{}", home_path, target::WORDLISTS_FOLDER);
            let parent_folder_created = match system::create_folder_if_not_exists(&parent_folder) {
                Ok(_) => format!("{}{}", home_path, target::WORDLISTS_FOLDER),
                Err(e) => {
                    println!("{}", e);
                    println!("Unable to create the folder, the wordlist will be saved in the current directory");
                    format!("./{}", filename)
                }
            };
            format!("{}{}", parent_folder_created, filename)
        }
        Err(_) => {
            println!("Unable to get the home directory, the wordlist will be saved in the current directory\n");
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
        Err(e) => {
            println!("{}", e);
            return Err(SystemError::UnableToCreateFile(filename, e.to_string()));
        }
    };

    Ok(file)
}
