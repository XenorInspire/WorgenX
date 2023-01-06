// Internal crates
use crate::system;

// This struct refers to a wordlist structure
struct WordlistConfig {
    pub numbers: bool,
    pub special_characters: bool,
    pub uppercase: bool,
    pub lowercase: bool,
    pub length: u64,
    pub wordlist_name: String,
    pub is_mask_mode_enabled: bool,
    pub mask: String,
}

// This is the main function of the wordlist generation module
pub fn main_wordlist_generation() {
    let mut again = String::from("y");

    while again.eq("y") {
        let wordlist_config = allocate_wordlist_config();
    }
}

// This function is charged to allocate the wordlist config structure
fn allocate_wordlist_config() -> WordlistConfig {
    let mut wordlist_config = WordlistConfig {
        numbers: false,
        special_characters: false,
        uppercase: false,
        lowercase: false,
        length: 0,
        wordlist_name: String::from(""),
        is_mask_mode_enabled: false,
        mask: String::from(""),
    };

    let mut choice;
    let mut is_option_chosen = false;

    while !is_option_chosen {
        println!("\nChoose what your wordlist is composed of :");
        println!("Uppercase letters (A-Z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                wordlist_config.uppercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Lowercase letters (a-z) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                wordlist_config.lowercase = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Numbers (0-9) ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                wordlist_config.numbers = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        println!("Special characters ? (y/n)");
        choice = system::get_user_choice_yn();
        match &*choice {
            "y" => {
                wordlist_config.special_characters = true;
                is_option_chosen = true;
            }
            _ => (),
        }

        if !is_option_chosen {
            println!("You must choose at least one character type !");
        }
    }

    println!("Do you want to enable the mask mode ? (y/n)");
    println!("The mask mode allows you to generate a wordlist based on a specific pattern.");
    choice = system::get_user_choice_yn();

    if choice.eq("y") {
        wordlist_config.is_mask_mode_enabled = true;
        println!("Enter the mask :");
        wordlist_config.mask = system::get_user_choice();
    } else {
        println!("Enter the length of the wordlist :");
        wordlist_config.length = system::get_user_choice_int();
    }

    println!("Enter the name of the wordlist :");
    let mut filename = String::new();

    while !system::is_valid_filename(filename.as_str()) {
        println!("Please enter the file name");
        filename = system::get_user_choice();
    }
    wordlist_config.wordlist_name = filename;

    wordlist_config
}
