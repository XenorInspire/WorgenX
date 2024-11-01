// Internal crates
use crate::{
    benchmark,
    error::SystemError,
    password::{self, PasswordConfig},
    system,
    wordlist::{self, WordlistConfig, WordlistValues},
};

#[cfg(target_family = "unix")]
use system::unix as target;

#[cfg(target_family = "windows")]
use system::windows as target;

// External crates
use std::{
    env,
    fs::{File, OpenOptions},
    sync::{Arc, Mutex},
    thread,
};

/// This function is charged to schedule in GUI mode the execution of the different features of the program according to the user's choices.
///
pub fn run() {
    loop {
        print_menu();
        let choice: String = system::get_user_choice();
        match &*choice {
            "0" => break,
            "1" => main_wordlist_generation(),
            "2" => main_passwd_generation(),
            "3" => main_benchmark(),
            _ => (),
        }
    }
    println!("Bye!");
}

/// This function is charged to display the header menu.
/// It is used to display the title of the program.
///
fn display_title() {
    for _ in 0..100 {
        print!("#");
    }
    println!();
}

/// This function is charged to display the menu.
/// It is used to display the WorgenX ASCII art and the different features of the program.
///
fn print_menu() {
    print!("\x1B[2J\x1B[1;1H"); // Clear the screen

    display_title();
    println!(
        r#"
      __        __                        __  __  _            __  __           ___       
      \ \      / /__  _ __ __ _  ___ _ __ \ \/ / | |__  _   _  \ \/ /___ _ __  / _ \ _ __ 
       \ \ /\ / / _ \| '__/ _` |/ _ \ '_ \ \  /  | '_ \| | | |  \  // _ \ '_ \| | | | '__|
        \ V  V / (_) | | | (_| |  __/ | | |/  \  | |_) | |_| |  /  \  __/ | | | |_| | |   
         \_/\_/ \___/|_|  \__, |\___|_| |_/_/\_\ |_.__/ \__, | /_/\_\___|_| |_|\___/|_|   
                          |___/                         |___/                             
"#
    );
    display_title();

    println!("\n1 : Create wordlist(s)");
    println!("2 : Generate random password(s)");
    println!("3 : Benchmark CPU");
    println!("0 : Exit WorgenX\n");
}

/// This is the main function of the random password generation feature.
///
fn main_passwd_generation() {
    let mut again: String = String::from("y");

    while again.eq("y") {
        let password_config: PasswordConfig = allocate_passwd_config_gui();
        let passwords: Vec<String> = password::generate_random_passwords(&password_config);

        println!("\nYou can find your password(s) below :\n\n{}", passwords.join("\n"));
        println!("\nDo you want to save the passwords in a file ? (y/n)");

        let choice: String = system::get_user_choice_yn();
        if choice.eq("y") {
            let mut file_result: Result<(File, String), SystemError> =
                saving_procedure(target::PASSWORDS_FOLDER);

            while file_result.is_err() {
                println!("{}", file_result.unwrap_err());
                println!("Do you want to try again ? (y/n)");
                if system::get_user_choice_yn().eq("n") {
                    return;
                }
                file_result = saving_procedure(target::PASSWORDS_FOLDER);
            }

            let (password_file, _) = file_result.unwrap();
            let shared_file: Arc<Mutex<File>> = Arc::new(Mutex::new(password_file));
            while let Err(e) =
                system::save_passwd_to_file(shared_file.clone(), passwords.join("\n"))
            {
                println!("\n{}", e);
                println!("Do you want to try again ? (y/n)");
                let choice: String = system::get_user_choice_yn();
                if choice.eq("n") {
                    println!("The passwords have not been saved");
                    return;
                }
            }
            println!("The passwords have been saved");
        }

        println!("\nDo you want to generate another password(s) ? (y/n)");
        again = system::get_user_choice_yn();
    }

    println!("\n");
}

/// This function is charged to allocate the password config structure from the user's choices in the GUI.
///
/// # Returns
///
/// The password config structure named PasswordConfig.
///
fn allocate_passwd_config_gui() -> PasswordConfig {
    let mut password_config: PasswordConfig = PasswordConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
        number_of_passwords: 0,
    };
    let mut is_option_chosen: bool = false;

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

/// This is the main function of the wordlist generation feature.
///
fn main_wordlist_generation() {
    let mut again: String = String::from("y");

    while again.eq("y") {
        let wordlist_values: WordlistValues = allocate_wordlist_config_gui();
        let wordlist_config: WordlistConfig = wordlist::build_wordlist_config(&wordlist_values);
        let nb_of_passwords: u64 = wordlist_config
            .dict
            .len()
            .pow(wordlist_config.mask_indexes.len() as u32)
            as u64;
        println!(
            "Estimated size of the wordlist: {}",
            system::get_estimated_size(nb_of_passwords, wordlist_config.formated_mask.len() as u64)
        );
        println!("Do you want to continue ? (y/n)");
        if system::get_user_choice_yn().eq("n") {
            return;
        }

        let mut file_result: Result<(File, String), SystemError> = saving_procedure(target::WORDLISTS_FOLDER);
        while file_result.is_err() {
            println!("{}", file_result.unwrap_err());
            println!("Do you want to try again ? (y/n)");
            if system::get_user_choice_yn().eq("n") {
                return;
            }
            file_result = saving_procedure(target::WORDLISTS_FOLDER);
        }

        println!("Wordlist generation started.");
        let (_, filename) = file_result.unwrap();
        if let Err(e) = wordlist::wordlist_generation_scheduler(
            &wordlist_config,
            nb_of_passwords,
            num_cpus::get_physical() as u8,
            &filename,
            false,
        ) {
            println!("{}", e);
            return;
        }

        println!("The wordlist has been saved in the file : {}", filename);
        println!("\nDo you want to generate another wordlist ? (y/n)");
        again = system::get_user_choice_yn();
    }
    println!("\n");
}

/// This function is charged to allocate the wordlist config structure from the user's choices.
///
/// # Returns
///
/// The wordlist values structure named WordlistValues.
///
fn allocate_wordlist_config_gui() -> WordlistValues {
    let mut wordlist_config: WordlistValues = WordlistValues {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        mask: String::new(),
        hash: String::new(),
    };
    let mut is_option_chosen: bool = false;

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

    println!("Do you want to hash the passwords of the wordlist ? (y/n)");
    if system::get_user_choice_yn().eq("y") {
        wordlist_config.hash = get_hash_choice();
    }

    println!("\nEnter the mask of the wordlist :");
    println!("For every character you want to be fixed, enter the character itself.");
    println!("For every character you want to be variable, enter a ?.");
    println!("If you want to specify the character '?' in the mask as a fixed character, enter '\\?'");

    let mut is_valid_mask: bool = false;
    while !is_valid_mask {
        wordlist_config.mask = system::get_user_choice();
        if wordlist_config.mask.is_empty() {
            println!("The mask cannot be empty !");
            continue;
        } else if !wordlist_config.mask.contains('?') {
            println!("The mask must contain at least one '?' !");
            continue;
        } else {
            println!("Do you want to validate the following mask : '{}' ? (y/n)", wordlist_config.mask);
            if system::get_user_choice_yn().eq("y") {
                is_valid_mask = true;
            }
        }
    }

    wordlist_config
}

/// This is the main function of the CPU benchmark feature.
/// It will start the benchmark after 5 seconds to let enough time for the user to read the message.
///
fn main_benchmark() {
    let mut again: String = String::from("y");

    while again.eq("y") {
        println!("The benchmark will start in 5 seconds...");
        thread::sleep(std::time::Duration::from_secs(5));
        match benchmark::load_cpu_benchmark(num_cpus::get_physical() as u8) {
            Ok(nb_of_passwords) => {
                println!("Your CPU has generated {} passwords in 1 minute", nb_of_passwords);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        println!("\nDo you want to run a new benchmark ? (y/n)");
        again = system::get_user_choice_yn();
    }
    println!("\n");
}

/// This function is charged to save the wordlist in a file with '\n' as a separator between each word.
/// This function is also charged to handle the creation of a backup file for the random passwords.
///
/// # Returns
///
/// A tuple containing the file and the file name as a string if the file has been created successfully.
/// Otherwise, it returns a SystemError.
///
pub fn saving_procedure(target: &str) -> Result<(File, String), SystemError> {
    println!("Please enter the file name to backup the wordlist :");
    let mut filename: String = system::get_user_choice();
    let mut result: Result<String, SystemError> = system::is_valid_path(filename.clone());
    while result.is_err() {
        println!("{}", result.unwrap_err());
        println!("Please enter a new file name:");
        filename = system::get_user_choice();
        result = system::is_valid_path(filename.clone());
    }

    let filename: String = match env::var(target::HOME_ENV_VAR) {
        Ok(home_path) => {
            let parent_folder: String = format!("{}{}", home_path, target);
            let parent_folder_created: String = match system::create_folder_if_not_exists(
                &parent_folder,
            ) {
                Ok(_) => format!("{}{}", home_path, target),
                Err(e) => {
                    println!("{}", e);
                    println!("Unable to create the folder, the file will be saved in the current directory");
                    format!("./{}", filename)
                }
            };
            format!("{}{}", parent_folder_created, filename)
        }
        Err(_) => {
            println!("Unable to get the home directory, the file will be saved in the current directory\n");
            format!("./{}", filename)
        }
    };

    let file: File = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename.clone())
    {
        Ok(file) => file,
        Err(e) => {
            println!("{}", e);
            return Err(SystemError::UnableToCreateFile(filename, e.to_string()));
        }
    };

    Ok((file, filename))
}

/// This function is charged to ask the hash algorithm from the user.
///
/// # Returns
///
/// The hash choice as a string. It returns an empty string if the user does not want to hash the passwords anymore.
///
fn get_hash_choice() -> String {
    let hash_choices: [&str; 14] = [
        "md5",
        "sha1",
        "sha224",
        "sha256",
        "sha384",
        "sha512",
        "sha3-224",
        "sha3-256",
        "sha3-384",
        "sha3-512",
        "blake2b-512",
        "blake2s-256",
        "whirlpool",
        "",
    ];

    loop {
        println!("Choose the hash algorithm you want to use :");
        for (i, hash) in hash_choices.iter().enumerate() {
            println!("{} : {}", i + 1, if hash.is_empty() { "None" } else { hash });
        }

        match system::get_user_choice().trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= hash_choices.len() => return hash_choices[n - 1].to_string(),
            _ => println!("Error: please specify a valid option"),
        }
    }
}
